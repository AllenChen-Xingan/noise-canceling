# 数据卡模板（023 · check_cards.py 按此校验）

> 每份数据集一张卡，文件名 `cards/<英文小写-连字符>.md`。14 个必填字段，格式固定为 `- **字段**：值`，字段名一字不改、顺序不变。查不到写「未查到」，不许空着。
> 场景键、设备键抄自 021 / 022 的 v0 草稿，正本在那两处；两边改了，本节随之改，`check_cards.py` 只认本节。

## 暂定场景键
> 正本：worktree `noise_canceling-scenes` 的 `docs/research/021_meeting-scenes/catalog.md`（v0 草稿，2026-11-03）。最后两个键是 mtgdata 请 scenes 加的，待确认。
`home-quiet` 书房 / 卧室 · `home-family` 家里客厅（家人 / 电视 / 小孩 / 宠物）· `open-office` 开放工位 · `phone-booth` 电话亭 / 专注舱 · `meeting-room` 会议室 · `cowork` 共享办公 · `cafe` 咖啡馆 · `canteen` 餐厅 / 食堂 · `transit-hub` 机场 / 车站候车厅 · `car-passenger` 车内乘客 · `rail-air-cabin` 高铁 / 飞机客舱 · `street-outdoor` 街边 / 户外 · `hotel-room` 酒店房间 · `library-classroom` 教室 / 图书馆 · `bus-metro-cabin` 公交 / 地铁 / 电车车厢（待加）· `other` 录音棚 / 实验室 / 采访间 / 未标（待加）

## 暂定设备键
> 正本：worktree `noise_canceling-macmic` 的 `docs/research/022_mac-mic/devices.md`（v0 草稿，2026-11-03）。`mac-unknown` 与 `non-laptop` 下 8 个子键是 mtgdata 请 macmic 加的，待确认。
`mba-builtin` MacBook Air 内置麦 · `mbp-builtin` MacBook Pro 内置麦 · `mac-intel-builtin` 2020 前 Intel Mac 内置麦 · `imac-builtin` · `studio-display` · `airpods-hfp` · `bt-hfp-generic` · `usb-headset` · `external-mic` · `iphone-continuity` · `laptop-unknown` 笔记本、型号未写 · `non-laptop` 不是笔记本（父类）· `mac-unknown` Mac 笔记本、型号未知（待加）· `phone` 手机 · `tablet` 平板 · `desktop` 台式机 / 摄像头麦 · `conf-device` 会议终端 · `mic-array` 研究用麦阵 · `close-talk` 近讲 / 领夹 / 头戴参考麦 · `studio` 录音棚麦 · `unknown` 设备未写（以上 8 个为 non-laptop 子键，待加）

## 卡片格式

```
# <数据集全名>

- **名称**：<全名（简称）>
- **链接**：<官方页 URL>
- **许可**：<许可原文名> → 结论：训练可 / 只评测 / 不可 / 未查到【来源标签】
- **大小**：<小时数 · GB · 条数>
- **场景键**：<键，逗号分隔>
- **设备键**：<键，逗号分隔>
- **笔记本段**：<有没有笔记本 / Mac 内置麦录的段 · 能不能按设备筛出来>
- **干净参考**：有 / 部分 / 没有（<说明>）
- **转写**：有 / 部分 / 没有（<有无时间戳>）
- **语种**：<语种>
- **语体**：<朗读 / 对话 / 会议；有无笑声、附和、情绪语音>
- **获取方式**：直接下载 / 注册 / 签协议 / 付费 / 已下架（<一句>）
- **与 D62 的关系**：<只列事实：它是真实录音还是回放 / 合成；开发阶段若用它评测会用到哪部分。不下结论>
- **出处**：<URL【官方文档 / 论文 / 论坛 / 推断】，多条用 ; 分隔>

## 备注
<自由文字：接近度、坑、怀疑派复核记录>
```

## 来源标签
【官方文档】【论文】【论坛 / 博客，个人说法】【推断】
