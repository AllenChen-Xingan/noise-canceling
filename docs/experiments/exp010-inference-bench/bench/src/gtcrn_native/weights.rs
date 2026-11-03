//! 扁平权重表（`<前缀>.bin` + `<前缀>.txt`）的读取与 BN 折叠。

use super::*;
use std::collections::HashMap;
use std::path::Path;

/// 扁平权重表：`<prefix>.bin` + `<prefix>.txt`。
pub struct Weights {
    pub(crate) data: Vec<f32>,
    pub(crate) index: HashMap<String, (Vec<usize>, usize, usize)>,
    /// 测试用：任何名字都返回这个常数填满的数组（不读文件）
    pub(crate) fake: Option<f32>,
}

impl Weights {
    pub fn load(prefix: &Path) -> Result<Self, Err> {
        let bin = std::fs::read(prefix.with_extension("bin"))?;
        let txt = std::fs::read_to_string(prefix.with_extension("txt"))?;
        let data: Vec<f32> = bin
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let mut index = HashMap::new();
        for line in txt.lines() {
            let p: Vec<&str> = line.split(' ').collect();
            if p.len() != 4 {
                return Err(format!("坏行：{line}").into());
            }
            let shape = p[1]
                .split(',')
                .map(str::parse)
                .collect::<Result<Vec<usize>, _>>()?;
            index.insert(p[0].to_string(), (shape, p[2].parse()?, p[3].parse()?));
        }
        if bin.len() % 4 != 0 {
            return Err(format!("权重 bin 长 {} 不是 4 的倍数", bin.len()).into());
        }
        let total: usize = index.values().map(|(_, _, n)| *n).sum();
        if total != data.len() {
            return Err(format!("txt 记的元素总数 {total} ≠ bin 里的 {}", data.len()).into());
        }
        Ok(Self {
            data,
            index,
            fake: None,
        })
    }

    #[cfg(test)]
    pub(crate) fn fake(v: f32) -> Self {
        Self {
            data: Vec::new(),
            index: HashMap::new(),
            fake: Some(v),
        }
    }

    pub(crate) fn get(&self, name: &str, numel: usize) -> Result<Vec<f32>, Err> {
        if let Some(v) = self.fake {
            return Ok(vec![v; numel]);
        }
        let (_, off, n) = self
            .index
            .get(name)
            .ok_or_else(|| format!("缺权重 {name}"))?;
        if *n != numel {
            return Err(format!("{name} 个数 {n} ≠ 期望 {numel}").into());
        }
        // txt 与 bin 对不上时返回错误，不 panic（自审 P2-5）
        self.data
            .get(*off..*off + *n)
            .map(<[f32]>::to_vec)
            .ok_or_else(|| format!("{name} 越界：偏移 {off} + {n} > {}", self.data.len()).into())
    }

    pub(crate) fn scalar(&self, name: &str) -> Result<f32, Err> {
        Ok(self.get(name, 1)?[0])
    }

    /// BN（推理态）折成逐通道 scale / shift。
    pub(crate) fn bn(&self, p: &str, c: usize) -> Result<(Vec<f32>, Vec<f32>), Err> {
        let g = self.get(&format!("{p}.weight"), c)?;
        let b = self.get(&format!("{p}.bias"), c)?;
        let m = self.get(&format!("{p}.running_mean"), c)?;
        let v = self.get(&format!("{p}.running_var"), c)?;
        let scale: Vec<f32> = g
            .iter()
            .zip(&v)
            .map(|(g, v)| g / (v + BN_EPS).sqrt())
            .collect();
        let shift = b
            .iter()
            .zip(&m)
            .zip(&scale)
            .map(|((b, m), s)| b - m * s)
            .collect();
        Ok((scale, shift))
    }
}
