# 差距表 · 014 真实录音公开数据集

> 2026-10-31。逐格对照 `must-know.md` 三条命门 + 接近度四项。

## 一、它们做到的（按我们要的格子）

| 我们要的格子 | 有 | 部分 | 没有 |
|---|---|---|---|
| 真实笔记本内置麦 | DNS4 / 5 dev（笔记本 / 台式机，未逐条标注）· ACE Chromebook 档 · DR-VCTK MacBook Air（回放） | REAL-M、NISQA LIVETALK（笔记本混手机） | 会议、CHiME、MISP 全系 |
| 真实咖啡馆 / 食堂 / 餐厅声场 | CHiME-3/4 CAF（平板）· ASPIRE（食堂，iPad）· ARTE（无语音）· DNS5 盲测（明列 café / babble，无标签） | ECHI、EasyCom、MIRaGe（人群声是音箱回放）· ACE 大堂（真人朗读 babble） | AMI / ICSI / AliMeeting / AISHELL-4 / NOTSOFAR（会议室） |
| 近距离邻座同时说话（D47） | CHiME-6、AliMeeting、DiPCo、ECHI、MMCSG（真实对话重叠）· DNS 个性化 dev 513 条邻座 | REAL-M（两人同读） | CHiME-4、DR-VCTK、ACE |
| 同步近讲参考 | CHiME-6（双耳）· AliMeeting（头戴，含 Test）· NOTSOFAR（train / dev）· AMI · ICSI · CHiME-4 ch0 · ECHI（仅限训练时用） | EasyCom（佩戴者无）· MCoRec（仅 train） | DNS、REAL-M、URGENT、AISHELL-4（录了未公开） |
| 时间戳转写 | CHiME-6 · AliMeeting · AISHELL-4 · NOTSOFAR · AMI · ICSI | DNS4 dev（无时间戳 WAcc 表）· CHiME-4（句级） | DNS 盲测（转了没公开）· URGENT · ECHI（未提） |
| 中文 | AliMeeting · AISHELL-4 · MISP · AISHELL-5 · RealMAN（回放） | DNS2 / 3（少量普通话） | 其余 |
| 可训练许可 | AMI · ICSI（爱丁堡版）· NOTSOFAR · DiPCo · DR-VCTK · VOiCES · BUT ReverbDB · RealMAN | Common Voice（CC0 但下载条款限制） | SA / NC / 自定义协议 / LDC 的全部 |

## 二、它们没做的

| 空格 | 没顾上做（机会）还是做不到（天堑） | 我们怎么办 |
|---|---|---|
| **笔记本内置麦 + 咖啡馆 + 近讲参考** | 没顾上：做法上不难（ECHI、ASPIRE 都证明可行），只是没人用笔记本做；隐私让公开发布更难 | 013 拍板 #4b 作者本人实录（只存本机）；**公开集补不上** |
| 中文 + 嘈杂公共场所 + 近讲 | 没顾上：中文真实集全是会议室 / 家居 / 车内 | AliMeeting 顶 D47 中文；咖啡馆中文只能靠模拟 + 作者实录 |
| 现代 MacBook 的实测设备响应 | 没顾上：只有 DR-VCTK 的旧 MacBook Air（16 kHz）和 DXOMARK 的图（无数据） | 013 拍板 #4a 回放重录 |
| DNS 盲测的转写 / 逐条场景标签 | 主办方转了但没放出 | 人工听选；DNS4 dev 用公开 WAcc 表 |
| 真实录音上的逐帧「你 / 别人」标签 | 没人专门做，但 CHiME-6 / AliMeeting 的近讲 + 时间戳转写能推出来 | 下一批做一个从转写生成逐帧标签的脚本（规则写死进仓，同 013 §1.1） |
