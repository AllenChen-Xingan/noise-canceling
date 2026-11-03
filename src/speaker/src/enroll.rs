//! 零设置注册器：不录音、不念句子，从会议里自己认出「主人」。
//!
//! 输入：判断回路挑出来的「清楚的单人片段」的声纹（每 0.5 s 至多一个），带时间、时长、电平。
//! 做法（003 §2.2，规则出处见 005 sources「家用说话人识别 / diart」的注册规则，代码自写）：
//! 1. **在线聚类**：与最近簇中心余弦 ≥ `assign_sim` 就并入，否则开新簇（最多 `max_clusters` 个，满了挤掉得分最低的）
//! 2. **得分 = 按最近程度衰减的累计说话时长 × 近讲系数**。近讲系数按电平：比最响的簇每低 6 dB，系数减半
//!    （主人离麦最近，邻座经过远场衰减；对策「会议开头别人先说话」的第一道）
//! 3. **锁定条件**（全满足）：首簇累计 ≥ `min_owner_s`；首簇得分 ≥ 次簇 × `lock_margin`；连续 `stable_updates` 次都是它；
//!    簇内平均相似度 ≥ `min_cohesion`（0.6，高于并入阈 0.5）；**只见过一个簇时要更长**（`single_cluster_min_s`）——
//!    开头只有邻座在说时，他是唯一的簇，最容易被错锁（对策第二道）
//! 4. **锁定后能改判**：另一簇得分超过主人 × `relock_ratio`、累计也够 `min_owner_s`，就改认它（对策第三道，纠错）
//! 5. 只存声纹向量；`reset()` 一键清空

#![allow(clippy::indexing_slicing)] // NRT: 判断回路里的注册器（C 档线程），不在降噪线程上跑

use crate::vecmath::{all_finite, cosine, l2_normalize};

#[derive(Clone, Copy, Debug)]
pub struct EnrollParams {
    pub assign_sim: f32,
    pub max_clusters: usize,
    pub min_owner_s: f32,
    pub single_cluster_min_s: f32,
    pub lock_margin: f32,
    pub stable_updates: u32,
    pub min_cohesion: f32,
    pub recency_half_life_s: f32,
    /// 电平每低这么多 dB，近讲系数减半
    pub level_halving_db: f32,
    pub relock_ratio: f32,
    /// 簇中心按至多这么多个样本平均（之后变成指数平均，允许声音慢慢变化）
    pub centroid_cap: f32,
}

impl Default for EnrollParams {
    /// 默认值取自 `vdn_types::desc`（`enroll.*`）。
    fn default() -> Self {
        use vdn_types::desc::*;
        Self {
            assign_sim: ENROLL_ASSIGN_SIM.default,
            max_clusters: ENROLL_MAX_CLUSTERS.default_count() as usize,
            min_owner_s: ENROLL_MIN_OWNER_S.default,
            single_cluster_min_s: ENROLL_SINGLE_CLUSTER_MIN_S.default,
            lock_margin: ENROLL_LOCK_MARGIN.default,
            stable_updates: ENROLL_STABLE_UPDATES.default_count(),
            // 必须高于并入阈 assign_sim，否则形同虚设（自审 P2-3）
            min_cohesion: ENROLL_MIN_COHESION.default,
            recency_half_life_s: ENROLL_RECENCY_HALF_LIFE_S.default,
            level_halving_db: ENROLL_LEVEL_HALVING_DB.default,
            relock_ratio: ENROLL_RELOCK_RATIO.default,
            centroid_cap: ENROLL_CENTROID_CAP.default,
        }
    }
}

impl EnrollParams {
    /// 每个参数都在描述表范围内（越界拒收）。穷举解构：加了字段没登记描述表，编译失败。
    pub fn validate(&self) -> Result<(), vdn_types::RangeError> {
        use vdn_types::desc::*;
        let EnrollParams {
            assign_sim,
            max_clusters,
            min_owner_s,
            single_cluster_min_s,
            lock_margin,
            stable_updates,
            min_cohesion,
            recency_half_life_s,
            level_halving_db,
            relock_ratio,
            centroid_cap,
        } = *self;
        ENROLL_ASSIGN_SIM.check(assign_sim)?;
        ENROLL_MAX_CLUSTERS.check_count(u32::try_from(max_clusters).unwrap_or(u32::MAX))?;
        ENROLL_MIN_OWNER_S.check(min_owner_s)?;
        ENROLL_SINGLE_CLUSTER_MIN_S.check(single_cluster_min_s)?;
        ENROLL_LOCK_MARGIN.check(lock_margin)?;
        ENROLL_STABLE_UPDATES.check_count(stable_updates)?;
        ENROLL_MIN_COHESION.check(min_cohesion)?;
        ENROLL_RECENCY_HALF_LIFE_S.check(recency_half_life_s)?;
        ENROLL_LEVEL_HALVING_DB.check(level_halving_db)?;
        ENROLL_RELOCK_RATIO.check(relock_ratio)?;
        ENROLL_CENTROID_CAP.check(centroid_cap)?;
        Ok(())
    }

    /// 夹进描述表范围（NaN 当默认值）；[`Enroller::new`] 先夹再用。
    pub fn clamped(self) -> Self {
        use vdn_types::desc::*;
        Self {
            assign_sim: ENROLL_ASSIGN_SIM.clamp(self.assign_sim),
            max_clusters: ENROLL_MAX_CLUSTERS
                .clamp_count(u32::try_from(self.max_clusters).unwrap_or(u32::MAX))
                as usize,
            min_owner_s: ENROLL_MIN_OWNER_S.clamp(self.min_owner_s),
            single_cluster_min_s: ENROLL_SINGLE_CLUSTER_MIN_S.clamp(self.single_cluster_min_s),
            lock_margin: ENROLL_LOCK_MARGIN.clamp(self.lock_margin),
            stable_updates: ENROLL_STABLE_UPDATES.clamp_count(self.stable_updates),
            min_cohesion: ENROLL_MIN_COHESION.clamp(self.min_cohesion),
            recency_half_life_s: ENROLL_RECENCY_HALF_LIFE_S.clamp(self.recency_half_life_s),
            level_halving_db: ENROLL_LEVEL_HALVING_DB.clamp(self.level_halving_db),
            relock_ratio: ENROLL_RELOCK_RATIO.clamp(self.relock_ratio),
            centroid_cap: ENROLL_CENTROID_CAP.clamp(self.centroid_cap),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cluster {
    /// 对外的簇中心 = normalize(sum)
    pub centroid: Vec<f32>,
    /// 未归一化的和（n 封顶后按指数平均衰减），簇中心由它归一化得到（自审 P2：不在归一化向量上做增量平均）
    pub sum: Vec<f32>,
    pub n: f32,
    /// 累计时长（秒，不衰减）
    pub dur_s: f32,
    /// 按最近程度衰减的累计时长
    pub decayed_s: f32,
    pub last_t: f64,
    /// 电平（dB）的滑动平均
    pub level_db: f32,
    /// 并入时与中心相似度的滑动平均；还没有任何并入时为 NaN（不能锁定）
    pub cohesion: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnrollState {
    Learning,
    Locked,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// 注册器每次观察的结果（与 vdn-types 的决策日志事件 `EnrollEvent { Locked, Relocked, Reset }` 不同名，接入时映射）
pub enum EnrollOutcome {
    None,
    Locked { cluster: usize },
    Relocked { from: usize, to: usize },
}

pub struct Enroller {
    p: EnrollParams,
    clusters: Vec<Cluster>,
    owner: Option<usize>,
    candidate: Option<usize>,
    streak: u32,
    now: f64,
}

impl Enroller {
    pub fn new(p: EnrollParams) -> Self {
        let p = p.clamped();
        Self {
            p,
            clusters: Vec::new(),
            owner: None,
            candidate: None,
            streak: 0,
            now: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.clusters.clear();
        self.owner = None;
        self.candidate = None;
        self.streak = 0;
    }

    pub fn state(&self) -> EnrollState {
        if self.owner.is_some() {
            EnrollState::Locked
        } else {
            EnrollState::Learning
        }
    }

    /// 主人声纹（锁定后）。
    pub fn owner(&self) -> Option<&[f32]> {
        self.owner.map(|i| self.clusters[i].centroid.as_slice())
    }

    pub fn owner_index(&self) -> Option<usize> {
        self.owner
    }

    pub fn clusters(&self) -> &[Cluster] {
        &self.clusters
    }

    /// 预置一个主人声纹（上一场会议存下的向量），直接锁定。
    pub fn preload(&mut self, owner: &[f32]) {
        self.reset();
        let mut c = owner.to_vec();
        l2_normalize(&mut c);
        self.clusters.push(Cluster {
            sum: c.clone(),
            centroid: c,
            n: self.p.centroid_cap * 0.25,
            dur_s: self.p.min_owner_s,
            decayed_s: self.p.min_owner_s,
            last_t: self.now,
            level_db: f32::NAN,
            cohesion: 1.0,
        });
        self.owner = Some(0);
    }

    fn score(&self, c: &Cluster, max_level: f32) -> f32 {
        let near = if c.level_db.is_finite() && max_level.is_finite() {
            2f32.powf((c.level_db - max_level) / self.p.level_halving_db)
        } else {
            1.0
        };
        c.decayed_s * near
    }

    pub fn scores(&self) -> Vec<f32> {
        let maxl = self
            .clusters
            .iter()
            .map(|c| c.level_db)
            .filter(|l| l.is_finite())
            .fold(f32::NEG_INFINITY, f32::max);
        self.clusters.iter().map(|c| self.score(c, maxl)).collect()
    }

    /// 喂一个清楚的单人片段。`t` 为片段终点（秒），`dur_s` 为片段里新增的人声时长，`level_db` 为片段人声电平。
    pub fn observe(&mut self, emb: &[f32], t: f64, dur_s: f32, level_db: f32) -> EnrollOutcome {
        if !all_finite(emb) || !dur_s.is_finite() {
            return EnrollOutcome::None;
        }
        // 衰减
        let dt = (t - self.now).max(0.0) as f32;
        self.now = t;
        let k = 0.5f32.powf(dt / self.p.recency_half_life_s);
        for c in &mut self.clusters {
            c.decayed_s *= k;
        }
        // 归类
        let best = self
            .clusters
            .iter()
            .enumerate()
            .map(|(i, c)| (i, cosine(&c.centroid, emb)))
            .max_by(|a, b| a.1.total_cmp(&b.1));
        let idx = match best {
            Some((i, s)) if s >= self.p.assign_sim => {
                let cap = self.p.centroid_cap;
                let c = &mut self.clusters[i];
                // n 未到上限：sum 是算术和；到上限后先按 (cap-1)/cap 衰减再加，等价于长度 cap 的指数平均
                if c.n >= cap {
                    let k = (cap - 1.0) / cap;
                    for x in c.sum.iter_mut() {
                        *x *= k;
                    }
                } else {
                    c.n += 1.0;
                }
                for (x, &e) in c.sum.iter_mut().zip(emb) {
                    *x += e;
                }
                c.centroid.copy_from_slice(&c.sum);
                l2_normalize(&mut c.centroid);
                c.dur_s += dur_s;
                c.decayed_s += dur_s;
                c.last_t = t;
                let la = 0.2;
                c.level_db = if c.level_db.is_finite() {
                    c.level_db + (level_db - c.level_db) * la
                } else {
                    level_db
                };
                c.cohesion = if c.cohesion.is_finite() {
                    c.cohesion + (s - c.cohesion) * la
                } else {
                    s
                };
                i
            }
            _ => {
                if self.clusters.len() >= self.p.max_clusters {
                    self.evict();
                }
                self.clusters.push(Cluster {
                    centroid: emb.to_vec(),
                    sum: emb.to_vec(),
                    n: 1.0,
                    dur_s,
                    decayed_s: dur_s,
                    last_t: t,
                    level_db,
                    cohesion: f32::NAN,
                });
                self.clusters.len() - 1
            }
        };
        let _ = idx;
        self.decide()
    }

    fn evict(&mut self) {
        let scores = self.scores();
        let victim = scores
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(*i) != self.owner)
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i);
        if let Some(v) = victim {
            self.clusters.remove(v);
            let fix = |o: &mut Option<usize>| {
                if let Some(x) = *o {
                    *o = if x == v {
                        None
                    } else if x > v {
                        Some(x - 1)
                    } else {
                        Some(x)
                    };
                }
            };
            fix(&mut self.owner);
            fix(&mut self.candidate);
        }
    }

    fn decide(&mut self) -> EnrollOutcome {
        let scores = self.scores();
        let mut order: Vec<usize> = (0..scores.len()).collect();
        order.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]));
        let Some(&top) = order.first() else {
            return EnrollOutcome::None;
        };
        let second = order.get(1).map(|&i| scores[i]).unwrap_or(0.0);
        if self.candidate == Some(top) {
            self.streak += 1;
        } else {
            self.candidate = Some(top);
            self.streak = 1;
        }
        let c = &self.clusters[top];
        match self.owner {
            None => {
                let need = if self.clusters.len() == 1 {
                    self.p.single_cluster_min_s
                } else {
                    self.p.min_owner_s
                };
                if c.dur_s >= need
                    && scores[top] >= second * self.p.lock_margin
                    && self.streak >= self.p.stable_updates
                    && c.cohesion.is_finite()
                    && c.cohesion >= self.p.min_cohesion
                {
                    self.owner = Some(top);
                    return EnrollOutcome::Locked { cluster: top };
                }
                EnrollOutcome::None
            }
            Some(o) if o != top => {
                if c.dur_s >= self.p.min_owner_s
                    && scores[top] >= scores[o] * self.p.relock_ratio
                    && self.streak >= self.p.stable_updates
                {
                    self.owner = Some(top);
                    return EnrollOutcome::Relocked { from: o, to: top };
                }
                EnrollOutcome::None
            }
            Some(_) => EnrollOutcome::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 描述表是正本：默认值逐条同表；越界 validate 拒收（报的是那一条）、clamped 夹到边界。
    #[test]
    fn enroll_params_follow_desc_table() {
        use vdn_types::desc::{self, ParamDesc};
        let d = EnrollParams::default();
        d.validate().unwrap();
        type Set = fn(&mut EnrollParams, f32);
        type Get = fn(&EnrollParams) -> f32;
        let fields: [(&ParamDesc, Get, Set); 9] = [
            (&desc::ENROLL_ASSIGN_SIM, |p| p.assign_sim, |p, v| p.assign_sim = v),
            (&desc::ENROLL_MIN_OWNER_S, |p| p.min_owner_s, |p, v| p.min_owner_s = v),
            (&desc::ENROLL_SINGLE_CLUSTER_MIN_S, |p| p.single_cluster_min_s, |p, v| p.single_cluster_min_s = v),
            (&desc::ENROLL_LOCK_MARGIN, |p| p.lock_margin, |p, v| p.lock_margin = v),
            (&desc::ENROLL_MIN_COHESION, |p| p.min_cohesion, |p, v| p.min_cohesion = v),
            (&desc::ENROLL_RECENCY_HALF_LIFE_S, |p| p.recency_half_life_s, |p, v| p.recency_half_life_s = v),
            (&desc::ENROLL_LEVEL_HALVING_DB, |p| p.level_halving_db, |p, v| p.level_halving_db = v),
            (&desc::ENROLL_RELOCK_RATIO, |p| p.relock_ratio, |p, v| p.relock_ratio = v),
            (&desc::ENROLL_CENTROID_CAP, |p| p.centroid_cap, |p, v| p.centroid_cap = v),
        ];
        for (dsc, get, set) in fields {
            assert_eq!(get(&d), dsc.default, "{} 默认值", dsc.key);
            for bad in [dsc.min - 0.5, dsc.max + 0.5] {
                let mut p = d;
                set(&mut p, bad);
                assert_eq!(p.validate().unwrap_err().desc.id, dsc.id, "{}", dsc.key);
                assert_eq!(get(&p.clamped()), dsc.clamp(bad), "{}", dsc.key);
            }
        }
        // 两个整数参数
        assert_eq!(d.max_clusters as u32, desc::ENROLL_MAX_CLUSTERS.default_count());
        assert_eq!(d.stable_updates, desc::ENROLL_STABLE_UPDATES.default_count());
        let p = EnrollParams { max_clusters: 0, ..d };
        assert_eq!(p.validate().unwrap_err().desc.id, desc::ENROLL_MAX_CLUSTERS.id);
        assert_eq!(p.clamped().max_clusters, 1);
        let p = EnrollParams { stable_updates: 1000, ..d };
        assert_eq!(p.validate().unwrap_err().desc.id, desc::ENROLL_STABLE_UPDATES.id);
        assert_eq!(p.clamped().stable_updates, 100);
    }

    fn unit(dim: usize, k: usize, jitter: f32, seed: &mut u32) -> Vec<f32> {
        let mut v = vec![0.0; dim];
        v[k] = 1.0;
        for x in v.iter_mut() {
            *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *x += jitter * (((*seed >> 8) as f32 / (1u32 << 24) as f32) - 0.5);
        }
        l2_normalize(&mut v);
        v
    }

    #[test]
    fn locks_on_loudest_longest_speaker() {
        let mut e = Enroller::new(EnrollParams::default());
        let mut s = 7;
        let mut t = 0.0;
        let mut ev = EnrollOutcome::None;
        // 主人（第 0 维，-20 dB）与邻座（第 1 维，-30 dB）轮流，主人说得多
        for i in 0..200 {
            t += 0.5;
            let (k, lvl) = if i % 3 == 0 { (1, -30.0) } else { (0, -20.0) };
            let r = e.observe(&unit(16, k, 0.3, &mut s), t, 0.5, lvl);
            if r != EnrollOutcome::None && ev == EnrollOutcome::None {
                ev = r;
            }
        }
        let o = e.owner().expect("应已锁定");
        assert!(o[0] > 0.9, "锁错人：{o:?}");
        assert!(matches!(ev, EnrollOutcome::Locked { .. }));
    }

    #[test]
    fn other_first_does_not_lock_before_owner_appears() {
        let mut e = Enroller::new(EnrollParams::default());
        let mut s = 3;
        let mut t = 0.0;
        // 前 25 s 只有邻座（单簇）——单簇要 30 s 才锁
        for _ in 0..50 {
            t += 0.5;
            e.observe(&unit(16, 1, 0.3, &mut s), t, 0.5, -30.0);
        }
        assert!(e.owner().is_none());
        // 主人开口，说得更多更响
        for i in 0..160 {
            t += 0.5;
            let (k, lvl) = if i % 4 == 0 { (1, -30.0) } else { (0, -20.0) };
            e.observe(&unit(16, k, 0.3, &mut s), t, 0.5, lvl);
        }
        assert!(e.owner().expect("应已锁定")[0] > 0.9);
    }

    #[test]
    fn relocks_when_wrong() {
        let mut e = Enroller::new(EnrollParams::default());
        let mut s = 5;
        let mut t = 0.0;
        // 邻座独白 40 s → 会锁错
        for _ in 0..80 {
            t += 0.5;
            e.observe(&unit(16, 1, 0.3, &mut s), t, 0.5, -30.0);
        }
        assert_eq!(e.owner_index(), Some(0));
        let mut relocked = false;
        for _ in 0..200 {
            t += 0.5;
            if let EnrollOutcome::Relocked { .. } =
                e.observe(&unit(16, 0, 0.3, &mut s), t, 0.5, -20.0)
            {
                relocked = true;
            }
        }
        assert!(relocked);
        assert!(e.owner().unwrap()[0] > 0.9);
    }
}
