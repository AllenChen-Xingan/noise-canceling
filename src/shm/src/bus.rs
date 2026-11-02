//! `Bus`（一块总线）与 `Ring`（其中一个环的视图）。本文件不含 unsafe（系统调用在 `wake.rs` / `region.rs`）。

use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use core::sync::atomic::{AtomicBool, AtomicU32};
#[cfg(all(unix, feature = "posix"))]
use std::ffi::CString;

use crate::layout::{self, Header, MAGIC, RINGS, RingHdr, VERSION};
use crate::region::Region;
use crate::wake;
use crate::{BusConfig, RingId, ShmError, WaitStrategy};

/// 一块总线。`Bus` 是 `Send + Sync`：可以 `&Bus` 分给生产线程与消费线程（遵守单生产者单消费者约定）。
/// Drop 只解映射；名字要创建方显式 `unlink()`。
pub struct Bus {
    pub(crate) region: Region,
    pub(crate) cap: usize,
    /// 是不是跨进程映射（决定 SHARED 标志：同一块内存两端必须一致）
    pub(crate) shared: bool,
    /// 本进程里 `wait` 用哪种方式等（不进共享内存）
    pub(crate) strategy: WaitStrategy,
    /// 唤醒系统调用出过不可恢复的错：本进程之后一律轮询
    pub(crate) wake_broken: AtomicBool,
    #[cfg(all(unix, feature = "posix"))]
    name: Option<CString>,
}

/// 名字规则：`/vdn-` 开头，总长 ≤ 31 字节（macOS PSHMNAMLEN），之后不再有 `/`，不含 NUL。
#[cfg(all(unix, feature = "posix"))]
fn checked_name(name: &str) -> Result<CString, ShmError> {
    let ok = name.starts_with("/vdn-")
        && name.len() > "/vdn-".len()
        && name.len() <= 31
        && !name.get(1..).is_some_and(|s| s.contains('/'));
    if !ok {
        return Err(ShmError::InvalidName);
    }
    CString::new(name).map_err(|_| ShmError::InvalidName)
}

fn checked_len(cfg: &BusConfig) -> Result<usize, ShmError> {
    if cfg.rate == 0 {
        return Err(ShmError::InvalidConfig("采样率为 0"));
    }
    if cfg.lead > cfg.capacity_frames {
        return Err(ShmError::InvalidConfig("lead 大于环容量"));
    }
    if cfg.denoise_block > cfg.capacity_frames {
        return Err(ShmError::InvalidConfig("降噪块长大于环容量"));
    }
    layout::total_bytes(cfg.capacity_frames)
        .ok_or(ShmError::InvalidConfig("容量为 0 或超过上限 2^24 帧"))
}

/// 构造（非实时）
impl Bus {
    /// 进程内总线（测试、单进程开发模式）。
    pub fn heap(cfg: BusConfig) -> Result<Self, ShmError> {
        let len = checked_len(&cfg)?;
        let bus = Self::wrap(Region::heap(len)?, cfg.capacity_frames as usize, false);
        bus.init_header(&cfg, len);
        Ok(bus)
    }

    /// 新建 POSIX 共享内存总线（降噪进程调）。同名已存在返回 `AlreadyExists`。
    pub fn create(name: &str, cfg: BusConfig) -> Result<Self, ShmError> {
        #[cfg(all(unix, feature = "posix"))]
        {
            let cname = checked_name(name)?;
            let len = checked_len(&cfg)?;
            let mut bus = Self::wrap(
                crate::region::posix::create(&cname, len)?,
                cfg.capacity_frames as usize,
                true,
            );
            bus.name = Some(cname);
            bus.init_header(&cfg, len);
            Ok(bus)
        }
        #[cfg(not(all(unix, feature = "posix")))]
        {
            let _ = (name, cfg);
            Err(ShmError::Unsupported)
        }
    }

    /// 打开已有总线（收音进程 / 虚拟麦克风调）。校验 magic · 版本 · 大小。
    pub fn open(name: &str) -> Result<Self, ShmError> {
        #[cfg(all(unix, feature = "posix"))]
        {
            let cname = checked_name(name)?;
            let region = crate::region::posix::open(&cname)?;
            let cap = validate(region.header(), region.len())?;
            let mut bus = Self::wrap(region, cap, true);
            bus.name = Some(cname);
            Ok(bus)
        }
        #[cfg(not(all(unix, feature = "posix")))]
        {
            let _ = name;
            Err(ShmError::Unsupported)
        }
    }

    /// 删掉名字（已映射的各方照常用到解映射为止）。创建方退出前调；heap 版是空操作。
    pub fn unlink(&self) -> Result<(), ShmError> {
        #[cfg(all(unix, feature = "posix"))]
        if let Some(n) = &self.name {
            return crate::region::posix::unlink(n);
        }
        Ok(())
    }

    /// 按名字删（清上次崩溃留下的同名对象）。不存在返回 `NotFound`。
    pub fn unlink_name(name: &str) -> Result<(), ShmError> {
        #[cfg(all(unix, feature = "posix"))]
        {
            crate::region::posix::unlink(&checked_name(name)?)
        }
        #[cfg(not(all(unix, feature = "posix")))]
        {
            let _ = name;
            Err(ShmError::Unsupported)
        }
    }

    /// 名字（heap 版为 None）。
    pub fn name(&self) -> Option<&str> {
        #[cfg(all(unix, feature = "posix"))]
        {
            self.name.as_ref().and_then(|n| n.to_str().ok())
        }
        #[cfg(not(all(unix, feature = "posix")))]
        {
            None
        }
    }

    pub fn config(&self) -> BusConfig {
        let c = &self.region.header().config;
        BusConfig {
            rate: c.rate.load(Relaxed),
            capacity_frames: c.capacity_frames.load(Relaxed),
            l_total: c.l_total.load(Relaxed),
            lead: c.lead.load(Relaxed),
            denoise_block: c.denoise_block.load(Relaxed),
        }
    }

    /// 会话 ID：`create` / `heap` 时随机生成，非 0。打开方记下它；重开后值不同 = 总线换了（降噪进程重启过）。
    pub fn session_id(&self) -> u64 {
        self.region.header().config.session.load(Relaxed)
    }

    /// 选 `wait` 的等法（只影响本进程本句柄）。默认 `WaitStrategy::Wake`。
    pub fn set_wait_strategy(&mut self, s: WaitStrategy) {
        self.strategy = s;
    }

    pub fn wait_strategy(&self) -> WaitStrategy {
        self.strategy
    }

    /// 本进程能不能用跨进程唤醒（macOS 14.4+ 且找到了系统函数）。不能时 `wait` 用 200 µs 轮询。
    pub fn wake_supported() -> bool {
        wake::init();
        wake::supported()
    }

    /// 只给测试用：让本进程假装系统不支持唤醒（验证退回轮询那条路）。全进程生效、不可撤销。
    #[doc(hidden)]
    pub fn force_wake_unsupported_for_test() {
        wake::init();
        wake::force_unsupported();
    }

    fn wrap(region: Region, cap: usize, shared: bool) -> Self {
        wake::init();
        Self {
            region,
            cap,
            shared,
            strategy: WaitStrategy::Wake,
            wake_broken: AtomicBool::new(false),
            #[cfg(all(unix, feature = "posix"))]
            name: None,
        }
    }

    fn init_header(&self, cfg: &BusConfig, len: usize) {
        let c = &self.region.header().config;
        c.version.store(VERSION, Relaxed);
        c.rate.store(cfg.rate, Relaxed);
        c.capacity_frames.store(cfg.capacity_frames, Relaxed);
        c.l_total.store(cfg.l_total, Relaxed);
        c.total_bytes.store(len as u64, Relaxed);
        c.lead.store(cfg.lead, Relaxed);
        c.denoise_block.store(cfg.denoise_block, Relaxed);
        c.session.store(new_session_id(), Relaxed);
        // 最后写 magic（Release）：打开方 Acquire 读到 MAGIC 时，上面各字段都已可见
        c.magic.store(MAGIC, Release);
    }
}

/// 随机、非 0 的会话 ID。std 的 `RandomState` 每个实例取一份系统随机种子；再混进时间与 pid。
fn new_session_id() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u32(std::process::id());
    if let Ok(t) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        h.write_u128(t.as_nanos());
    }
    h.finish().max(1)
}

/// 打开方校验头部；返回每环容量。
#[cfg(all(unix, feature = "posix"))]
fn validate(h: &Header, mapped: usize) -> Result<usize, ShmError> {
    let c = &h.config;
    match c.magic.load(Acquire) {
        0 => return Err(ShmError::NotReady),
        MAGIC => {}
        _ => return Err(ShmError::BadMagic),
    }
    let found = c.version.load(Relaxed);
    if found != VERSION {
        return Err(ShmError::BadVersion {
            found,
            expected: VERSION,
        });
    }
    let cap = c.capacity_frames.load(Relaxed);
    let header = c.total_bytes.load(Relaxed);
    let expect = layout::total_bytes(cap).ok_or(ShmError::BadSize {
        header,
        mapped: mapped as u64,
    })?;
    // macOS 会把共享内存对象按页取整，所以映射 ≥ 头部记的大小即可
    if header != expect as u64 || expect > mapped {
        return Err(ShmError::BadSize {
            header,
            mapped: mapped as u64,
        });
    }
    if c.rate.load(Relaxed) == 0 {
        return Err(ShmError::InvalidConfig("对端写的采样率为 0"));
    }
    if c.lead.load(Relaxed) > cap || c.denoise_block.load(Relaxed) > cap {
        return Err(ShmError::InvalidConfig(
            "对端写的 lead 或降噪块长大于环容量",
        ));
    }
    Ok(cap as usize)
}

/// 实时可调：不分配、不加锁、不 panic；除 `notify` 的一次非阻塞唤醒外不做系统调用。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Bus {
    /// 某个环的视图。
    pub fn ring(&self, id: RingId) -> Ring<'_> {
        let h = self.region.header();
        let [raw, processed, out] = &h.rings;
        let (i, hdr) = match id {
            RingId::Raw => (0, raw),
            RingId::Processed => (1, processed),
            RingId::Out => (2, out),
        };
        let slots = self
            .region
            .slots(RINGS * self.cap)
            .and_then(|all| all.get(i * self.cap..(i + 1) * self.cap))
            .unwrap_or(&[]);
        Ring { hdr, slots }
    }

    /// 生产方：push 完调一次，叫醒在 `wait(id, ..)` 上睡着的消费方。
    /// 实时可调（规约 §4 A 档唯一例外）：一次原子加 + 至多一次非阻塞系统调用（`os_sync_wake_by_address_any`），
    /// 不分配、不加锁、不阻塞；没人在等时系统立刻返回。系统不支持时只做原子加。
    pub fn notify(&self, id: RingId) {
        let w = &self.ring(id).hdr.prod.wake;
        // Release：消费方 Acquire 读到新值时，之前 push 的样本都可见
        w.fetch_add(1, Release);
        wake::wake_one(w, self.shared);
    }

    /// 降噪进程每处理完一块调一次。
    pub fn beat(&self) {
        self.region.header().beat.heartbeat.fetch_add(1, Release);
    }

    /// 心跳计数；收音进程看它多久没动来决定是否切原声。
    pub fn heartbeat(&self) -> u64 {
        self.region.header().beat.heartbeat.load(Acquire)
    }

    /// 送原声用的总延迟（48 kHz 样本）。
    pub fn l_total(&self) -> u32 {
        self.region.header().config.l_total.load(Relaxed)
    }

    pub fn rate(&self) -> u32 {
        self.region.header().config.rate.load(Relaxed)
    }

    /// 每个环的容量（帧）。
    pub fn capacity(&self) -> usize {
        self.cap
    }
}

/// 一个环的视图（借自 `Bus`，复制便宜）。
#[derive(Clone, Copy)]
pub struct Ring<'a> {
    pub(crate) hdr: &'a RingHdr,
    slots: &'a [AtomicU32],
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Ring<'_> {
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// 已用量；对端写坏索引时夹在 [0, capacity]。
    fn used(&self, w: u64, r: u64) -> u64 {
        w.wrapping_sub(r).min(self.slots.len() as u64)
    }

    /// 环里从逻辑位置 `pos` 起的槽（回绕后接上开头）。
    fn from(&self, pos: u64) -> impl Iterator<Item = &AtomicU32> {
        let cap = self.slots.len() as u64;
        let start = if cap == 0 { 0 } else { (pos % cap) as usize };
        let (head, tail) = self.slots.split_at_checked(start).unwrap_or((&[], &[]));
        tail.iter().chain(head)
    }

    /// 生产方：写入 `src`，返回写进去的样本数；写不下的部分丢掉并计溢出。
    pub fn push(&self, src: &[f32]) -> usize {
        let cap = self.slots.len() as u64;
        let w = self.hdr.prod.write.load(Relaxed); // 只有自己写
        let r = self.hdr.cons.read.load(Acquire); // 与消费方 read 的 Release 配对：它读完的槽才能覆盖
        let free = cap - self.used(w, r);
        let n = (src.len() as u64).min(free) as usize;
        for (slot, x) in self.from(w).zip(src.iter().take(n)) {
            slot.store(x.to_bits(), Relaxed);
        }
        self.hdr.prod.write.store(w.wrapping_add(n as u64), Release); // 发布样本
        let lost = src.len() - n;
        if lost > 0 {
            self.hdr.prod.overflow.fetch_add(lost as u64, Relaxed);
        }
        n
    }

    /// 消费方：读出最多 `dst.len()` 个样本，返回读到的个数；不够时 `dst` 余下部分不动（欠载由调用方计）。
    pub fn pop(&self, dst: &mut [f32]) -> usize {
        let r = self.hdr.cons.read.load(Relaxed); // 只有自己写
        let w = self.hdr.prod.write.load(Acquire); // 与生产方 write 的 Release 配对：看得到它写的样本
        let n = (dst.len() as u64).min(self.used(w, r)) as usize;
        for (x, slot) in dst.iter_mut().take(n).zip(self.from(r)) {
            *x = f32::from_bits(slot.load(Relaxed));
        }
        self.hdr.cons.read.store(r.wrapping_add(n as u64), Release); // 归还槽
        n
    }

    /// 消费方：读出最多 `dst.len()` 个样本，不够的部分**填 0 并计欠载**，返回读到的个数（HAL 输出等「必须交满一块」的地方用）。
    pub fn pop_or_zero(&self, dst: &mut [f32]) -> usize {
        let n = self.pop(dst);
        let missing = dst.len() - n;
        if missing > 0 {
            for x in dst.iter_mut().skip(n) {
                *x = 0.0;
            }
            self.note_underrun(missing);
        }
        n
    }

    /// 消费方：丢掉最多 `n` 个样本（重新对齐用），返回丢掉的个数。
    pub fn discard(&self, n: usize) -> usize {
        let r = self.hdr.cons.read.load(Relaxed);
        let w = self.hdr.prod.write.load(Acquire);
        let n = (n as u64).min(self.used(w, r));
        self.hdr.cons.read.store(r.wrapping_add(n), Release);
        n as usize
    }

    /// 可读样本数（消费方看是下限，生产方看是上限）。
    pub fn available(&self) -> usize {
        // 先读 read 再读 write：两个索引都只增不减，这样第三方线程读到的 write ≥ read，不会回绕成「满」（自审 P2）
        let r = self.hdr.cons.read.load(Acquire);
        let w = self.hdr.prod.write.load(Acquire);
        self.used(w, r) as usize
    }

    /// 可写空间（生产方看是下限）。
    pub fn free_space(&self) -> usize {
        self.slots.len() - self.available()
    }

    /// 消费方记欠载（想要 n 个样本没拿到）。
    pub fn note_underrun(&self, n: usize) {
        self.hdr.cons.underrun.fetch_add(n as u64, Relaxed);
    }

    /// 累计溢出样本数。
    pub fn overflows(&self) -> u64 {
        self.hdr.prod.overflow.load(Relaxed)
    }

    /// 累计欠载样本数。
    pub fn underruns(&self) -> u64 {
        self.hdr.cons.underrun.load(Relaxed)
    }

    /// 读清溢出计数（判断回路按窗口读，规约 §3.7.4b）。
    pub fn take_overflows(&self) -> u64 {
        self.hdr.prod.overflow.swap(0, Relaxed)
    }

    /// 读清欠载计数。
    pub fn take_underruns(&self) -> u64 {
        self.hdr.cons.underrun.swap(0, Relaxed)
    }

    /// 消费方记缺口（因落后主动丢掉的积压，例如 `discard` 之后调），与欠载分开。
    pub fn note_gap(&self, n: usize) {
        self.hdr.cons.gaps.fetch_add(n as u64, Relaxed);
    }

    /// 累计缺口样本数。
    pub fn gaps(&self) -> u64 {
        self.hdr.cons.gaps.load(Relaxed)
    }

    /// 读清缺口计数。
    pub fn take_gaps(&self) -> u64 {
        self.hdr.cons.gaps.swap(0, Relaxed)
    }

    /// `wait` 退回轮询的累计次数（系统不支持或调用出错）。
    pub fn wait_fallbacks(&self) -> u64 {
        self.hdr.cons.wait_fallbacks.load(Relaxed)
    }
}
