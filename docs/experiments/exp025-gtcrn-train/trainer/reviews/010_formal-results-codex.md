# 评审记录 010：正式训练读数（feat/trainer-formal-results）· codex

- 对象：`origin/main...HEAD` 的 diff（README 新节、smoke_reports/ 读数与漂移 JSON 及生成脚本、执行单前提 6）
- 输出与日志：主仓 `_tmp/20261116-b19/codex-formalres/`

## 第 1 轮（gpt-6-astra，medium，只读沙箱）：P0×0 P1×0 P2×3

| 编号 | 内容 | 处理 |
|---|---|---|
| 第 1 轮 #1 | 主干漂移的分母含官方固定的 ERB 矩阵，不是「只比可训练参数」 | 属实。改为只取 `requires_grad` 为真的主干参数，重新生成；A、B 各离起点 3.2–3.8%（原 3.2–3.7%），A、B 之间 0.3–0.5%（不变） |
| 第 1 轮 #2 | JSON 的 B − A 先舍入再相减，与 README 三处差 0.0001 | 属实（README 原就是未舍入相减）。脚本改为未舍入相减，JSON 现与 README 一致 |
| 第 1 轮 #3 | 梯度范数最大值只是已记录步上的最大值 | 属实。README 与 JSON 字段名都注明「已记录步（第 1 步与每 20 步）」 |

另：脚本加了一条断言，核 B 的模型配置与重造初值用的参数相同（codex 抽查里说未核实）。
