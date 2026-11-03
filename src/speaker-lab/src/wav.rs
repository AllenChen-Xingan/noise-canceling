//! 16 kHz 单声道 WAV 读写。

use std::path::Path;

use crate::{R, SR};

pub(crate) fn read_wav(p: &Path) -> R<Vec<f32>> {
    let mut r = hound::WavReader::open(p)?;
    let s = r.spec();
    if s.sample_rate != SR || s.channels != 1 {
        return Err(format!("{}: 只收 16 kHz 单声道", p.display()).into());
    }
    Ok(match s.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let k = 1.0 / (1u64 << (s.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|v| v.map(|x| x as f32 * k))
                .collect::<Result<_, _>>()?
        }
    })
}

pub(crate) fn write_wav(p: &Path, x: &[f32]) -> R<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SR,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(p, spec)?;
    for &v in x {
        w.write_sample(v)?;
    }
    w.finalize()?;
    Ok(())
}
