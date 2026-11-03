//! 一块按 64 字节对齐、初值全零的内存：进程内堆（`heap`）或 POSIX 共享内存映射（`create` / `open`）。
//! 本 crate 的 unsafe 全在这个文件里：分配 / 映射，以及把裸指针转成 `&Header` 与 `&[AtomicU32]`。

#![allow(unsafe_code)] // 白名单 crate（metadata.vdn.unsafe = true）；每个 unsafe 块带 SAFETY

#[cfg(feature = "loom")]
pub(crate) use model::Region;
#[cfg(not(feature = "loom"))]
pub(crate) use real::*;

#[cfg(not(feature = "loom"))]
mod real {
    use crate::sync::AtomicU32;
    use core::ptr::NonNull;
    use std::alloc::Layout;

    use crate::ShmError;
    use crate::layout::{HEADER_BYTES, Header, SLOT_BYTES};

    enum Kind {
        Heap(Layout),
        #[cfg(all(unix, feature = "posix"))]
        Mmap,
    }

    pub(crate) struct Region {
        ptr: NonNull<u8>,
        len: usize,
        kind: Kind,
    }

    // SAFETY: Region 只经 `header()` / `slots()` 给出对原子量的共享引用；原子量本身 Sync，
    // 裸指针指向的内存在 Region 活着时一直有效，Drop 只发生一次。
    unsafe impl Send for Region {}
    // SAFETY: 同上——所有访问都经原子操作，没有经 &Region 的非原子写。
    unsafe impl Sync for Region {}

    impl Region {
        /// 进程内：按 64 字节对齐分配 `len` 字节并清零。
        pub(crate) fn heap(len: usize) -> Result<Self, ShmError> {
            if len < HEADER_BYTES {
                return Err(ShmError::InvalidConfig("区域小于头部"));
            }
            let layout = Layout::from_size_align(len, 64)
                .map_err(|_| ShmError::InvalidConfig("区域大小无法对齐到 64 字节"))?;
            // SAFETY: layout 大小 ≥ HEADER_BYTES > 0。
            let raw = unsafe { std::alloc::alloc_zeroed(layout) };
            let ptr = NonNull::new(raw).ok_or(ShmError::OutOfMemory)?;
            Ok(Self {
                ptr,
                len,
                kind: Kind::Heap(layout),
            })
        }

        pub(crate) fn len(&self) -> usize {
            self.len
        }

        pub(crate) fn header(&self) -> &Header {
            // SAFETY: 构造时保证 len ≥ HEADER_BYTES、起点按 64 对齐（堆 layout 对齐 64；mmap 按页对齐）；
            // 内存初值全零或由创建方写入，Header 全由原子量组成，任何位模式都合法；
            // 生命周期绑在 &self 上，Region 活着映射就在。
            unsafe { &*self.ptr.as_ptr().cast::<Header>() }
        }

        /// 头部之后的 `count` 个样本槽；区域放不下返回 None。
        pub(crate) fn slots(&self, count: usize) -> Option<&[AtomicU32]> {
            let need = count.checked_mul(SLOT_BYTES)?.checked_add(HEADER_BYTES)?;
            if need > self.len {
                return None;
            }
            // SAFETY: [HEADER_BYTES, need) 在区域内（上面检查过）；HEADER_BYTES 是 4 的倍数（layout.rs 编译期断言），
            // 起点按 64 对齐，所以槽按 AtomicU32 对齐；AtomicU32 任何位模式都合法；只读共享引用，写都经原子操作。
            Some(unsafe {
                core::slice::from_raw_parts(
                    self.ptr.as_ptr().add(HEADER_BYTES).cast::<AtomicU32>(),
                    count,
                )
            })
        }
    }

    impl Drop for Region {
        fn drop(&mut self) {
            match self.kind {
                // SAFETY: ptr 由同一个 layout 的 alloc_zeroed 得来，只释放这一次。
                Kind::Heap(layout) => unsafe { std::alloc::dealloc(self.ptr.as_ptr(), layout) },
                #[cfg(all(unix, feature = "posix"))]
                Kind::Mmap => {
                    // SAFETY: ptr / len 正是 mmap 返回与传入的值，只解映射这一次。失败也无可挽回，忽略返回值。
                    let _ = unsafe { libc::munmap(self.ptr.as_ptr().cast(), self.len) };
                }
            }
        }
    }

    #[cfg(all(unix, feature = "posix"))]
    pub(crate) mod posix {
        use super::{Kind, Region};
        use crate::ShmError;
        use crate::layout::HEADER_BYTES;
        use core::ptr::NonNull;
        use std::ffi::CString;

        fn errno() -> i32 {
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
        }

        fn os(op: &'static str) -> ShmError {
            ShmError::Os { op, errno: errno() }
        }

        /// 把整个 fd 映射成可读写共享内存，然后关 fd（映射不依赖 fd）。
        fn map(fd: libc::c_int, len: usize) -> Result<Region, ShmError> {
            // SAFETY: fd 是刚打开的共享内存对象，len > 0；返回值下面检查 MAP_FAILED。
            let p = unsafe {
                libc::mmap(
                    core::ptr::null_mut(),
                    len,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    fd,
                    0,
                )
            };
            let err = (p == libc::MAP_FAILED).then(|| os("mmap"));
            // SAFETY: fd 归本函数调用方转交，只关这一次。
            unsafe { libc::close(fd) };
            if let Some(e) = err {
                return Err(e);
            }
            let ptr = NonNull::new(p.cast::<u8>()).ok_or(ShmError::Os {
                op: "mmap",
                errno: 0,
            })?;
            Ok(Region {
                ptr,
                len,
                kind: Kind::Mmap,
            })
        }

        /// 新建（已存在则报 `AlreadyExists`），大小 `len`，权限 0600（只有同一用户能打开；发版前换 Mach 服务交接，009 待人拍板 3）。
        pub(crate) fn create(name: &CString, len: usize) -> Result<Region, ShmError> {
            let off =
                libc::off_t::try_from(len).map_err(|_| ShmError::InvalidConfig("区域过大"))?;
            // SAFETY: name 是以 NUL 结尾的 C 字符串；shm_open 是可变参数函数，mode 按 c_uint 传。
            let fd = unsafe {
                libc::shm_open(
                    name.as_ptr(),
                    libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
                    0o600 as libc::c_uint,
                )
            };
            if fd < 0 {
                return Err(if errno() == libc::EEXIST {
                    ShmError::AlreadyExists
                } else {
                    os("shm_open")
                });
            }
            // SAFETY: fd 有效；新建对象的 ftruncate 只做这一次（macOS 只允许一次）。
            if unsafe { libc::ftruncate(fd, off) } != 0 {
                let e = os("ftruncate");
                // SAFETY: 清理刚建的对象：关 fd、删名字。
                unsafe {
                    libc::close(fd);
                    libc::shm_unlink(name.as_ptr());
                }
                return Err(e);
            }
            map(fd, len).inspect_err(|_| {
                // SAFETY: 映射失败时删掉刚建的名字，别留垃圾。
                unsafe { libc::shm_unlink(name.as_ptr()) };
            })
        }

        /// 打开已有的；映射整个对象（大小以 fstat 为准，头部校验由调用方做）。
        pub(crate) fn open(name: &CString) -> Result<Region, ShmError> {
            // SAFETY: name 是以 NUL 结尾的 C 字符串；不带 O_CREAT 时不读 mode。
            let fd = unsafe { libc::shm_open(name.as_ptr(), libc::O_RDWR) };
            if fd < 0 {
                return Err(if errno() == libc::ENOENT {
                    ShmError::NotFound
                } else {
                    os("shm_open")
                });
            }
            // SAFETY: stat 是纯数据结构，全零是合法初值；fstat 只写它。
            let mut st: libc::stat = unsafe { core::mem::zeroed() };
            // SAFETY: fd 有效，st 可写。
            if unsafe { libc::fstat(fd, &mut st) } != 0 {
                let e = os("fstat");
                // SAFETY: 只关这一次。
                unsafe { libc::close(fd) };
                return Err(e);
            }
            let len = usize::try_from(st.st_size).unwrap_or(0);
            if len < HEADER_BYTES {
                // 创建方还没 ftruncate 完
                // SAFETY: 只关这一次。
                unsafe { libc::close(fd) };
                return Err(ShmError::NotReady);
            }
            map(fd, len)
        }

        pub(crate) fn unlink(name: &CString) -> Result<(), ShmError> {
            // SAFETY: name 是以 NUL 结尾的 C 字符串。
            if unsafe { libc::shm_unlink(name.as_ptr()) } != 0 {
                return Err(if errno() == libc::ENOENT {
                    ShmError::NotFound
                } else {
                    os("shm_unlink")
                });
            }
            Ok(())
        }
    }
}

/// loom 版：头部与样本槽分开分配（loom 的原子量不能从一块全零内存里「认出来」），接口与真版相同。
#[cfg(feature = "loom")]
mod model {
    use crate::ShmError;
    use crate::layout::{HEADER_BYTES, Header, SLOT_BYTES};
    use crate::sync::AtomicU32;

    pub(crate) struct Region {
        header: Box<Header>,
        slots: Box<[AtomicU32]>,
        len: usize,
    }

    impl Region {
        pub(crate) fn heap(len: usize) -> Result<Self, ShmError> {
            let n = len
                .checked_sub(HEADER_BYTES)
                .ok_or(ShmError::InvalidConfig("区域小于头部"))?
                / SLOT_BYTES;
            Ok(Self {
                header: Box::new(Header::new_zeroed()),
                slots: (0..n).map(|_| AtomicU32::new(0)).collect(),
                len,
            })
        }

        pub(crate) fn len(&self) -> usize {
            self.len
        }

        pub(crate) fn header(&self) -> &Header {
            &self.header
        }

        pub(crate) fn slots(&self, count: usize) -> Option<&[AtomicU32]> {
            self.slots.get(..count)
        }
    }
}
