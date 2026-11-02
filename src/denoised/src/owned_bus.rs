//! 建总线一方的守卫（构造期、退出期用，非实时）。
// NRT: 开锁文件、建 / 删共享内存都是系统调用，只在启动和退出时调
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]

use vdn_shm::Bus;

/// 建总线一方的守卫：
/// - 每个总线名配一把**文件锁**（`$TMPDIR/vdn-bus-lock_<名>`，`File::try_lock`）。同名第二个建方拿不到锁，
///   直接报「正被占用」，不会删掉别人在用的总线；进程被 SIGKILL 时内核也会放锁
/// - 拿到锁之后才清上次异常退出留下的同名对象（`unlink_name`），这时它一定没人在建
/// - Drop 时 unlink 一次（只一次），然后才放锁
pub struct OwnedBus {
    bus: Bus,
    unlinked: std::sync::atomic::AtomicBool,
    _lock: std::fs::File,
}

impl OwnedBus {
    pub fn create(name: &str, cfg: vdn_shm::BusConfig) -> Result<Self, String> {
        vdn_control::validate_bus_name(name).map_err(|e| e.to_string())?;
        let lock_path =
            std::env::temp_dir().join(format!("vdn-bus-lock{}", name.replace('/', "_")));
        let lock = std::fs::File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&lock_path)
            .map_err(|e| format!("开锁文件 {}：{e}", lock_path.display()))?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(format!(
                    "总线 {name} 正被另一个降噪进程（或 chain-test）使用；换个 --bus 名字，或先停掉它"
                ));
            }
            Err(std::fs::TryLockError::Error(e)) => {
                return Err(format!("锁 {}：{e}", lock_path.display()));
            }
        }
        // 有锁 = 没有活着的建方；同名对象只可能是上次异常退出的残留
        let _ = Bus::unlink_name(name);
        let bus = Bus::create(name, cfg).map_err(|e| format!("建总线 {name}：{e}"))?;
        Ok(Self {
            bus,
            unlinked: std::sync::atomic::AtomicBool::new(false),
            _lock: lock,
        })
    }

    /// 删名字（只删一次；已删过返回 Ok）。映射本身仍可用到 drop。
    pub fn unlink(&self) -> Result<(), vdn_shm::ShmError> {
        if self
            .unlinked
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            return Ok(());
        }
        self.bus.unlink()
    }
}

impl std::ops::Deref for OwnedBus {
    type Target = Bus;
    fn deref(&self) -> &Bus {
        &self.bus
    }
}

impl Drop for OwnedBus {
    fn drop(&mut self) {
        let _ = self.unlink();
    }
}
