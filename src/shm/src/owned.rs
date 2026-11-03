//! `OwnedBus`：建总线一方的守卫（构造期、退出期用，非实时）。
//!
//! 问题：建方异常退出会留下同名共享内存对象；下一个建方要先删它，但不能删掉**别的活着的建方**正在用的那个。
//! 做法：每个总线名配一把文件锁（`flock`，进程死了内核自动放锁）。
//! - 锁文件：`$TMPDIR/vdn-shm<名字里的 / 换成 _>.lock`，例如 `/vdn-out` → `$TMPDIR/vdn-shm_vdn-out.lock`。
//!   名字已经过 `/vdn-` 前缀 · ≤ 31 字节 · 不再含 `/` 的检查，所以不会逃出 `$TMPDIR`
//! - 拿到锁 = 没有活着的建方；这时才删残留、再 `create`
//! - 拿不到锁：返回 `ShmError::Busy`，什么都不删
//! - Drop：先删名字（只删一次），再解映射、放锁
//!
//! 注意：锁只在**建方之间**互斥；打开方（`Bus::open`）不碰锁。两个建方的 `$TMPDIR` 不同时锁不住彼此
//! （同一用户的普通进程 `$TMPDIR` 相同）。

#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // NRT: 开锁文件、建 / 删共享内存只在启动和退出时调

use core::ops::Deref;
use core::sync::atomic::{AtomicBool, Ordering::AcqRel};
use std::fs::{File, TryLockError};
use std::path::PathBuf;

use crate::{Bus, BusConfig, ShmError};

/// 建总线一方持有的总线。`Deref` 到 `Bus`；Drop 时删名字。
pub struct OwnedBus {
    bus: Bus,
    unlinked: AtomicBool,
    lock_path: PathBuf,
    // 最后 drop：名字删完才放锁
    _lock: File,
}

impl OwnedBus {
    /// 拿锁 → 清残留 → 建总线。同名正被别的活着的建方持有时返回 `ShmError::Busy`。
    pub fn create(name: &str, cfg: BusConfig) -> Result<Self, ShmError> {
        // 先走一遍名字检查（Bus::create 也会查；这里提前，免得用坏名字建锁文件）
        if let Err(e) = Bus::open(name)
            && e == ShmError::InvalidName
        {
            return Err(e);
        }
        let lock_path = Self::lock_path_for(name);
        let lock = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&lock_path)
            .map_err(|e| ShmError::Os {
                op: "open(锁文件)",
                errno: e.raw_os_error().unwrap_or(0),
            })?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Err(ShmError::Busy),
            Err(TryLockError::Error(e)) => {
                return Err(ShmError::Os {
                    op: "flock(锁文件)",
                    errno: e.raw_os_error().unwrap_or(0),
                });
            }
        }
        // 有锁 = 没有活着的建方；同名对象只可能是上次异常退出的残留
        let _ = Bus::unlink_name(name);
        let bus = Bus::create(name, cfg)?;
        Ok(Self {
            bus,
            unlinked: AtomicBool::new(false),
            lock_path,
            _lock: lock,
        })
    }

    /// 某个名字对应的锁文件路径（不检查名字）。
    pub fn lock_path_for(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("vdn-shm{}.lock", name.replace('/', "_")))
    }

    /// 本总线的锁文件路径。
    pub fn lock_path(&self) -> &std::path::Path {
        &self.lock_path
    }

    /// 删名字（只删一次；已删过返回 Ok）。已映射的各方照常用到解映射为止。
    pub fn unlink(&self) -> Result<(), ShmError> {
        if self.unlinked.swap(true, AcqRel) {
            return Ok(());
        }
        self.bus.unlink()
    }

    /// 拿可变引用（例如 `set_wait_strategy`）。
    pub fn bus_mut(&mut self) -> &mut Bus {
        &mut self.bus
    }
}

impl Deref for OwnedBus {
    type Target = Bus;
    fn deref(&self) -> &Bus {
        &self.bus
    }
}

impl Drop for OwnedBus {
    fn drop(&mut self) {
        let _ = self.unlink();
        // 锁文件本身留着（删了会和下一个建方竞争：它可能已经开了同一路径的旧 inode）
    }
}
