//! BNNS Graph（Accelerate，macOS 15+）后端：C FFI 手写声明。
//! 链接名照本机 SDK 27.0 的 `bnns_graph.h` 的 `__asm__` 标注（带 `_v2` 的几个，旧名已从 TBD 删掉，019 notes/bnns-graph.md）。
//! 做法：单线程编译；工作区按页对齐一次性分配；参数用裸指针；缓存两组 A/B 来回换指针，不拷贝。
use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;

use crate::{Backend, ModelKind};

#[repr(C)]
#[derive(Clone, Copy)]
struct Opaque {
    data: *mut c_void,
    size: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Arg {
    data_ptr: *mut c_void,
    data_ptr_size: usize,
}
type MsgFn = unsafe extern "C" fn(u32, *const c_char, *const c_char, *mut c_void);

#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn BNNSGraphCompileOptionsMakeDefault() -> Opaque;
    fn BNNSGraphCompileOptionsDestroy(o: Opaque);
    fn BNNSGraphCompileOptionsSetTargetSingleThread(o: Opaque, v: bool);
    fn BNNSGraphCompileOptionsSetMessageLogCallback(o: Opaque, f: MsgFn, extra: *mut c_void);
    fn BNNSGraphCompileOptionsSetMessageLogMask(o: Opaque, mask: u32);
    #[link_name = "\u{1}_BNNSGraphCompileFromFile_v2"]
    fn BNNSGraphCompileFromFile(file: *const c_char, function: *const c_char, o: Opaque) -> Opaque;
    fn BNNSGraphGetArgumentCount(g: Opaque, function: *const c_char) -> usize;
    fn BNNSGraphGetArgumentNames(g: Opaque, function: *const c_char, n: usize, names: *mut *const c_char) -> i32;
    fn BNNSGraphGetArgumentPosition(g: Opaque, function: *const c_char, arg: *const c_char) -> usize;
    fn BNNSGraphContextMake(g: Opaque) -> Opaque;
    #[link_name = "\u{1}_BNNSGraphContextDestroy_v2"]
    fn BNNSGraphContextDestroy(c: Opaque);
    fn BNNSGraphContextSetArgumentType(c: Opaque, t: u32) -> i32;
    #[link_name = "\u{1}_BNNSGraphContextGetWorkspaceSize_v2"]
    fn BNNSGraphContextGetWorkspaceSize(c: Opaque, function: *const c_char) -> usize;
    #[link_name = "\u{1}_BNNSGraphContextExecute_v2"]
    fn BNNSGraphContextExecute(c: Opaque, function: *const c_char, n: usize, args: *mut Arg, ws_size: usize, ws: *mut c_void)
    -> i32;
}

unsafe extern "C" fn on_msg(level: u32, msg: *const c_char, loc: *const c_char, _: *mut c_void) {
    let m = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    let l = if loc.is_null() { "".into() } else { unsafe { CStr::from_ptr(loc) }.to_string_lossy() };
    eprintln!("[bnns {level}] {m} {l}");
}

pub struct Bnns {
    graph: Opaque,
    ctx: Opaque,
    ws: *mut c_void,
    ws_size: usize,
    ws_layout: std::alloc::Layout,
    args: [Vec<Arg>; 2],
    xin: Vec<f32>,
    yout: Vec<f32>,
    _caches: Vec<Vec<f32>>,
    kind: ModelKind,
    flip: usize,
    pub compile_ms: f64,
    pub info: String,
}

impl Bnns {
    /// `mlmodelc`：Core ML 编译产物目录；`cache_sizes`：各缓存元素数（顺序同 `kind.cache_names()`）。
    pub fn open(mlmodelc: &str, kind: ModelKind, cache_sizes: &[usize], single_thread: bool) -> Result<Self, String> {
        unsafe {
            let t = std::time::Instant::now();
            let o = BNNSGraphCompileOptionsMakeDefault();
            BNNSGraphCompileOptionsSetTargetSingleThread(o, single_thread);
            BNNSGraphCompileOptionsSetMessageLogCallback(o, on_msg, ptr::null_mut());
            BNNSGraphCompileOptionsSetMessageLogMask(o, 2 | 4 | 8); // 不支持 · 警告 · 错误
            let path = CString::new(mlmodelc).unwrap();
            let graph = BNNSGraphCompileFromFile(path.as_ptr(), ptr::null(), o);
            BNNSGraphCompileOptionsDestroy(o);
            if graph.data.is_null() {
                return Err("BNNSGraphCompileFromFile 失败（看上面的 [bnns] 日志）".into());
            }
            let compile_ms = t.elapsed().as_secs_f64() * 1e3;
            let ctx = BNNSGraphContextMake(graph);
            if ctx.data.is_null() {
                return Err("BNNSGraphContextMake 失败".into());
            }
            BNNSGraphContextSetArgumentType(ctx, 0); // BNNSGraphArgumentTypePointer
            let ws_size = BNNSGraphContextGetWorkspaceSize(ctx, ptr::null());
            let page = libc::sysconf(libc::_SC_PAGESIZE) as usize;
            let ws_layout = std::alloc::Layout::from_size_align(ws_size.div_ceil(page) * page + page, page).unwrap();
            let ws = std::alloc::alloc_zeroed(ws_layout) as *mut c_void;

            let n = BNNSGraphGetArgumentCount(graph, ptr::null());
            let mut names = vec![ptr::null::<c_char>(); n];
            BNNSGraphGetArgumentNames(graph, ptr::null(), n, names.as_mut_ptr());
            let names: Vec<String> = names.iter().map(|p| CStr::from_ptr(*p).to_string_lossy().into_owned()).collect();
            let pos = |s: &str| -> Result<usize, String> {
                let c = CString::new(s).unwrap();
                let p = BNNSGraphGetArgumentPosition(graph, ptr::null(), c.as_ptr());
                if p == usize::MAX { Err(format!("图里没有参数 {s}（有：{names:?}）")) } else { Ok(p) }
            };
            let per = kind.per_frame();
            let mut xin = vec![0f32; per];
            let mut yout = vec![0f32; per];
            let (cin, cout) = kind.cache_names();
            // 两组缓存：caches[2*i] 是 A 组，caches[2*i+1] 是 B 组
            let mut caches: Vec<Vec<f32>> = cache_sizes.iter().flat_map(|&s| [vec![0f32; s], vec![0f32; s]]).collect();
            let mut args = [vec![Arg { data_ptr: ptr::null_mut(), data_ptr_size: 0 }; n], vec![Arg { data_ptr: ptr::null_mut(), data_ptr_size: 0 }; n]];
            let (pi, po) = (pos(kind.input_name())?, pos(kind.output_name())?);
            for k in 0..2 {
                args[k][pi] = Arg { data_ptr: xin.as_mut_ptr().cast(), data_ptr_size: per * 4 };
                args[k][po] = Arg { data_ptr: yout.as_mut_ptr().cast(), data_ptr_size: per * 4 };
                for i in 0..cin.len() {
                    let (a, b) = (pos(&cin[i])?, pos(&cout[i])?);
                    let sz = cache_sizes[i] * 4;
                    args[k][a] = Arg { data_ptr: caches[2 * i + k].as_mut_ptr().cast(), data_ptr_size: sz };
                    args[k][b] = Arg { data_ptr: caches[2 * i + 1 - k].as_mut_ptr().cast(), data_ptr_size: sz };
                }
            }
            if let Some(i) = args[0].iter().position(|a| a.data_ptr.is_null()) {
                return Err(format!("参数 {} 没有配上缓冲", names[i]));
            }
            Ok(Self {
                graph,
                ctx,
                ws,
                ws_size,
                ws_layout,
                args,
                xin,
                yout,
                _caches: caches,
                kind,
                flip: 0,
                compile_ms,
                info: format!("{{\"workspace_bytes\":{ws_size},\"single_thread\":{single_thread},\"args\":{n}}}"),
            })
        }
    }
}

impl Backend for Bnns {
    fn run(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), String> {
        self.kind.pack_input(input, &mut self.xin);
        let rc = unsafe {
            BNNSGraphContextExecute(self.ctx, ptr::null(), self.args[self.flip].len(), self.args[self.flip].as_mut_ptr(), self.ws_size, self.ws)
        };
        if rc != 0 {
            return Err(format!("BNNSGraphContextExecute rc={rc}"));
        }
        self.flip ^= 1;
        self.kind.unpack_output(&self.yout, output);
        Ok(())
    }
}

impl Drop for Bnns {
    fn drop(&mut self) {
        unsafe {
            BNNSGraphContextDestroy(self.ctx);
            libc::free(self.graph.data); // 头文件：未指定输出路径时由调用方 free
            std::alloc::dealloc(self.ws.cast(), self.ws_layout);
        }
    }
}
