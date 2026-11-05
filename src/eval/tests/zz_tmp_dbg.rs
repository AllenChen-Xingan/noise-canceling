use vdn_eval::analysis::masks;
use vdn_eval::synth::{NoiseKind, SceneSpec, TalkerSpec, render};
use vdn_eval::system::{SystemKind, SystemSpec, run};
use vdn_eval::verifiers::{Ctx, Params, eval};
#[test]
fn dbg() {
    let c = render(&SceneSpec { dur_s: 10.0, seed: 7, owner: Some(TalkerSpec { level_dbov: -26.0, f0_lo: 100.0, f0_hi: 160.0, tonal: true }),
        owner_start_s: 1.0, pause_s: (0.3, 0.9), interferers: Vec::new(), noise: NoiseKind::Pink, snr_db: 5.0, tail_silence_s: 0.0 });
    let spec = SystemSpec { kind: SystemKind::Ref, ..Default::default() };
    let r = run(&spec, &c.mix).unwrap();
    eprintln!("mix {} raw {} out {} lat {}", c.mix.len(), r.raw.len(), r.out.len(), r.latency);
    let m = masks(&c.truth, &c.mix, 0.03);
    let ctx = Ctx { clip: &c, out: &r.out, run: Some(&r), masks: &m, noise_out: None, ci: true };
    let s = eval("V-LAT", &ctx, &Params::default());
    eprintln!("{s} {:?} {}", s.extra, s.note);
}
