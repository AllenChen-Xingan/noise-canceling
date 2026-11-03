//! Pipeline 报出的处理图：级串照描述、模型节点与正在跑的 `Model::name()` 比对、权重的制品内容 ID 进图 ID（#68）。

use vdn_engine::testing::{GAIN, Gain, Identity};
use vdn_engine::{Engine, Model, ModelError, Pipeline, SafetyConfig, stages};
use vdn_types::{ChainMismatch, ContentId, ModelId, ProcessingGraph};

mod common;
use common::SR;

/// 输出 = 输入，名字可指定（冒充某个真模型的名字）。
struct Named(&'static str);
impl Model for Named {
    fn name(&self) -> &'static str {
        self.0
    }
    fn sample_rate(&self) -> u32 {
        Identity.sample_rate()
    }
    fn hop(&self) -> usize {
        Identity.hop()
    }
    fn latency_samples(&self) -> usize {
        0
    }
    fn reset(&mut self) {}
    fn process(&mut self, i: &[f32], o: &mut [f32]) -> Result<(), ModelError> {
        o.copy_from_slice(i);
        Ok(())
    }
}

const W1: ContentId = ContentId(0x1111);
const W2: ContentId = ContentId(0x2222);

#[test]
fn pipeline_reports_graph_with_model_node() {
    let id = ModelId::Gtcrn;
    let eng = Engine::new(Named(id.as_str()), SafetyConfig::off());
    let p = Pipeline::with_stages(eng, SR, None, &[&GAIN], stages![Gain(1.0)]).unwrap();
    let g = p.graph(id, W1).unwrap();
    assert_eq!(g.stages, &[&GAIN]);
    assert!(g.spectral.is_empty());
    assert_eq!((g.model, g.artifact), (id, W1));
    // 不带级的构造对应只有模型节点的图
    let plain = Pipeline::new(Engine::new(Named(id.as_str()), SafetyConfig::off()), SR).unwrap();
    assert!(plain.stage_kinds().is_empty());
    assert_eq!(
        plain.graph(id, W1).unwrap().content_id(),
        ProcessingGraph::bare(id, W1).content_id()
    );
}

/// 只换模型版本（同名模型换了权重）：图 ID 必须变。
#[test]
fn only_artifact_changes_graph_id() {
    let id = ModelId::FastEnhancerM;
    let p = Pipeline::new(Engine::new(Named(id.as_str()), SafetyConfig::off()), SR).unwrap();
    let a = p.graph(id, W1).unwrap();
    let b = p.graph(id, W2).unwrap();
    assert_eq!(a.model, b.model);
    assert_ne!(a.content_id(), b.content_id());
}

/// 模型节点名与正在跑的模型不一致 → 报错（构造期），错误写出两边的名字。
#[test]
fn model_name_mismatch_is_rejected() {
    let p = Pipeline::new(Engine::new(Identity, SafetyConfig::off()), SR).unwrap();
    for id in ModelId::ALL {
        assert_eq!(
            p.graph(id, W1).unwrap_err(),
            ChainMismatch::ModelName {
                expected: id.as_str(),
                actual: Identity.name()
            }
        );
    }
    let p = Pipeline::new(
        Engine::new(Named(ModelId::Gtcrn.as_str()), SafetyConfig::off()),
        SR,
    )
    .unwrap();
    assert!(p.graph(ModelId::FastEnhancerM, W1).is_err());
}
