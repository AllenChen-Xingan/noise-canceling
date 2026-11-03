//! `vdn` 的命令行参数（clap 定义）：只描述参数，不做事。

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use vdn_models::ModelKind;

#[derive(Parser)]
#[command(name = "vdn", version, about = "Mac 开源实时语音降噪 · P1 离线原型")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// WAV → 逐帧流式降噪 + 三件保命 → WAV（32-bit float）
    Denoise(DenoiseArgs),
    /// 单线程每帧耗时（口径同 exp001：热身后计 N 步）
    Bench(BenchArgs),
    /// 列输入 / 输出设备与采样率（不开流、不弹权限框）
    Devices,
    /// 单进程实时开发模式：麦克风 → Pipeline → 扬声器（会开麦、出声）
    Run(RunArgs),
    /// 无设备链路测试：进程内三线程 + 真 POSIX 共享内存，按 V1 / V2 / V3 判据打印并给退出码
    ChainTest(ChainArgs),
    /// V7 延迟判据：GTCRN 链路 N 次 × T 秒，空闲一组 + 后台压力一组；可扫环余量找最小无欠载值
    V7(V7Args),
    /// 按 V1 判据检查一个 48 kHz WAV（V4：两个真进程写出的文件）：应是测试正弦延迟 L_total
    ChainCheck(ChainCheckArgs),
}

#[derive(Args)]
pub struct V7Args {
    #[command(flatten)]
    pub m: ModelOpts,
    /// 每组跑几次
    #[arg(long, default_value_t = 10)]
    pub runs: usize,
    /// 每次几秒
    #[arg(long, default_value_t = 10.0, value_parser = vdn_control::parse_seconds)]
    pub seconds: f64,
    /// 环余量（逗号分隔可扫多个，如 96,144,192）；默认取 vdn-control 的默认值
    #[arg(long)]
    pub leads: Option<String>,
    /// 压力组的忙线程数；默认 = 核数 − 3（三个链路线程各占一个核）
    #[arg(long)]
    pub stress: Option<usize>,
    /// 只跑空闲组
    #[arg(long)]
    pub idle_only: bool,
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_BLOCK)]
    pub block: usize,
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_DENOISE_BLOCK)]
    pub denoise_block: usize,
    /// 结果写成 JSON
    #[arg(long)]
    pub json: Option<PathBuf>,
}

#[derive(Args)]
pub struct ChainCheckArgs {
    pub wav: PathBuf,
    /// 期望的总延迟（48 kHz 样本；vdn-capture 启动时打印）
    #[arg(long)]
    pub l_total: usize,
    /// 从第几秒起判（重接之后那段）
    #[arg(long, default_value_t = 0.0, value_parser = vdn_control::parse_seconds)]
    pub from: f64,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct RunMode {
    /// 直通（testing::Identity，不需要模型权重）
    #[arg(long)]
    pub passthrough: bool,
    /// 降噪（真模型）
    #[arg(long)]
    pub denoise: bool,
}

#[derive(Args)]
pub struct RunArgs {
    #[command(flatten)]
    pub mode: RunMode,
    #[command(flatten)]
    pub m: ModelOpts,
    /// 跑多少秒；不给就一直跑到 Ctrl-C
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    pub seconds: Option<f64>,
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_BLOCK)]
    pub block: usize,
    /// 环余量（48 kHz 样本）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_LEAD)]
    pub lead: usize,
    #[arg(long)]
    pub input_device: Option<String>,
    #[arg(long)]
    pub output_device: Option<String>,
    /// 输出增益（dB，−60..=+20，默认 0）：只给 V5 试听用，放大小声的麦克风
    #[arg(long, default_value_t = 0.0, allow_negative_numbers = true)]
    pub gain_db: f64,
    /// 把送进耳机的那一路同时写进这个 WAV（48 kHz 单声道 32 位浮点），结束后可用耳机回放
    #[arg(long)]
    pub record: Option<PathBuf>,
    /// V5 机器辅助判据：不给 --seconds 就跑 5 s，结束打印输入 / 输出电平与判定；不过时退出码 1
    #[arg(long)]
    pub check: bool,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ChainModeArg {
    Passthrough,
    Denoise,
}

#[derive(Args)]
pub struct ChainArgs {
    #[arg(long, value_enum, default_value = "passthrough")]
    pub mode: ChainModeArg,
    #[command(flatten)]
    pub m: ModelOpts,
    /// 第几秒让降噪线程退出（测 V2）
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    pub kill_denoiser_at: Option<f64>,
    #[arg(long, default_value_t = 5.0, value_parser = vdn_control::parse_seconds)]
    pub seconds: f64,
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_BLOCK)]
    pub block: usize,
    /// 环余量（48 kHz 样本）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_LEAD)]
    pub lead: usize,
    /// 降噪块长（48 kHz 样本，须整除模型 hop × 3）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_DENOISE_BLOCK)]
    pub denoise_block: usize,
    /// 共享内存名（默认 /vdn-chaintest）
    #[arg(long)]
    pub bus: Option<String>,
    /// 心跳超时（ms）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_HEARTBEAT_MS)]
    pub heartbeat_ms: u32,
    /// 三个线程不提实时优先级
    #[arg(long)]
    pub no_rt: bool,
    /// 后台压力：再起这么多个忙线程
    #[arg(long, default_value_t = 0)]
    pub stress: usize,
    /// 降噪线程预热块数
    #[arg(long, default_value_t = 64)]
    pub warmup_blocks: usize,
    /// 虚拟麦克风比送音端晚多少样本读 Out（默认 1.5 块）
    #[arg(long)]
    pub hal_offset: Option<usize>,
    /// 第几秒经控制通道切到直通（测运行中切换不断声）
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    pub switch_mode_at: Option<f64>,
    /// 把输入 / 输出写成 WAV（48 kHz）方便人听：<前缀>-in.wav / <前缀>-out.wav
    #[arg(long)]
    pub dump: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ModelArg {
    Gtcrn,
    #[value(name = "fastenhancer-m")]
    FastEnhancerM,
}

impl From<ModelArg> for ModelKind {
    fn from(m: ModelArg) -> Self {
        match m {
            ModelArg::Gtcrn => ModelKind::Gtcrn,
            ModelArg::FastEnhancerM => ModelKind::FastEnhancerM,
        }
    }
}

#[derive(Args)]
pub struct ModelOpts {
    /// 用哪个模型
    #[arg(long, value_enum, default_value = "gtcrn")]
    pub model: ModelArg,
    /// 模型文件；不给就按 环境变量 VDN_GTCRN_ONNX / VDN_FASTENHANCER_ONNX → ~/vdn-data/models/… → ./resources/… 查找
    #[arg(long)]
    pub model_path: Option<PathBuf>,
}

#[derive(Args)]
pub struct DenoiseArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    #[command(flatten)]
    pub m: ModelOpts,
    /// 三件保命全关（输出 = 模型原样输出，对拍用）
    #[arg(long)]
    pub no_safety: bool,
    /// 关「出事送原声」
    #[arg(long)]
    pub no_fallback: bool,
    /// 关衰减下限（同时关快开慢关）
    #[arg(long)]
    pub no_floor: bool,
    /// 关快开慢关（只留固定的衰减下限）
    #[arg(long)]
    pub no_fast_open: bool,
    /// 衰减下限（dB）：停顿时背景最多降这么多
    #[arg(long, default_value_t = vdn_types::desc::FLOOR_DB.default, allow_hyphen_values = true)]
    pub floor_db: f32,
    /// 有声时的原声混回（dB），须 ≥ floor-db
    #[arg(long, default_value_t = vdn_types::desc::OPEN_DB.default, allow_hyphen_values = true)]
    pub open_db: f32,
    /// 人声停后保持放开多久（ms）
    #[arg(long, default_value_t = vdn_types::desc::HANGOVER_S.default * 1e3)]
    pub hangover_ms: f32,
    /// 收回下限的时间常数（ms）
    #[arg(long, default_value_t = vdn_types::desc::RELEASE_S.default * 1e3)]
    pub release_ms: f32,
    /// 模型每次 process 的截止时间（ms），超时那一帧送原声；默认不设
    #[arg(long)]
    pub deadline_ms: Option<f32>,
    /// 开头先喂多少个零样本（对拍 GTCRN 时用 256，与 exp003 Python 口径一致）
    #[arg(long, default_value_t = 0)]
    pub lead_in: usize,
    /// 故障注入（实验用）：逗号分隔的 `<hop>:<nan|error|explode|stall=<ms>>`
    #[arg(long)]
    pub inject: Option<String>,
    /// 把统计与每帧耗时写成 JSON
    #[arg(long)]
    pub json: Option<PathBuf>,
}

#[derive(Args)]
pub struct BenchArgs {
    #[command(flatten)]
    pub m: ModelOpts,
    /// 输入 WAV（16 kHz 单声道），不够长就循环
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long, default_value_t = 200)]
    pub warmup: usize,
    #[arg(long, default_value_t = 6000)]
    pub steps: usize,
    /// 结果 JSON
    #[arg(long)]
    pub json: Option<PathBuf>,
}
