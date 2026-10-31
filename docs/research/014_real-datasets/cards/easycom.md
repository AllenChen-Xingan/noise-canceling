### EasyCom（2021，Meta / Facebook Reality Labs）
- **做什么**：AR 眼镜佩戴者在嘈杂餐厅中的对话增强，多模态
- **录音设备与距离**：一人戴 AR 眼镜：6 ch 阵列（ch1–4 镜框麦 + ch5–6 入耳双耳麦），48 kHz/32 bit；其他人（服务员除外）戴单声道头戴麦（论文称 Rode）+ 仿真眼镜；同桌，距离未给 m 数（论文 §III）
- **环境**：约 6×7×3 m 房间，RT60 645 ms；10 个扬声器播放空间不相关的餐厅噪声，约 71 dB SPL（README 与 §III）→ 回放噪声，部分真实
- **说话人数与重叠**：每场约 30 min，3–5 人 + 一名“服务员”，点餐、拼图、I spy 游戏，含重叠
- **语言**：英语
- **参考**：近讲 有（头戴；眼镜佩戴者与服务员无头戴，佩戴者用眼镜双耳麦）· 转写 有（以视频帧号计时，20 fps）· 说话人标签 有
- **大小**：5 h 18 min 11 s 高质量 + 带错误数据共约 6.6 h；GB 未查到
- **许可**：CC BY-NC 4.0 → 仅非商业 → 只评测
- **获取**：直接下载 · github.com/facebookresearch/EasyComDataset（Releases）
- **接近度**：3.5/5 · 餐厅噪声 + 邻座多人，接近 D42/D47；但噪声为扬声器回放、设备为眼镜阵列
- **用途建议**：sim-to-real 校准（71 dB SPL 餐厅场景的 SNR/谱/混响）、本机回归集、人耳听测；不训练
- **出处**：https://github.com/facebookresearch/EasyComDataset 【官方】；arXiv:2107.04174v2 §III, §III-B 【论文】

<details><summary>另一路的卡片（easycom-D-cafe.md，怀疑派已修正；与上文冲突处见 ../decision.md 附录「卡片冲突裁定」）</summary>

#### EasyCom（2021，Meta Reality Labs）
- **做什么**：AR 眼镜在嘈杂环境里辅助对话。
- **录音设备与距离**：佩戴者戴 AR 眼镜（镜框 4 麦 + 2 支入耳麦）；邻座约 1 m 级【推算】。
- **环境**：10 通道扬声器播放空间不相关的餐厅噪声，固定在约 71 dB SPL——**噪声是回放的**，对话是真人现场。
- **说话人数与重叠**：多人围桌（含一名「服务员」），有重叠。
- **语言**：英语。
- **参考**：近讲 有，**但只给非佩戴者**（佩戴者和服务员都没有头戴麦）——设备佩戴者自己的语音没有近讲参考 · 转写 有 · 说话人标签 有。
- **大小**：5 h 18 min。
- **许可**：CC BY-NC 4.0 · 结论：只评测。
- **获取**：直接下载（GitHub releases）https://github.com/facebookresearch/EasyComDataset
- **接近度**：3/5。
- **用途建议**：回归；听测；参照 71 dB SPL 做电平校准。只存本机。
- **出处**：https://github.com/facebookresearch/EasyComDataset 【官方】· arXiv 2107.04174 【论文】

</details>
