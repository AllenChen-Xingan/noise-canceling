# 问题 7 · 嘈杂多人场景下怎么温和地提示（UX）

## 一句话结论

主流会议软件只做两类「你那边有问题」提示：**说话时却处于静音**（Teams 2017 起、Webex 40.11 起、Zoom 6.2 起），以及**你在制造回声**（Meet 2021 起）。它们都是会议窗口里的非模态横条或小红点，**不走系统通知**。「你那边噪音大」这类主动提示：在 Zoom、Teams、Webex 上未查到；Meet 只给了一个声音指示圈，其中「一次性提示每场会最多一次」。苹果 HIG 明确要求：只告知信息时不用 alert，同一件事不许重复通知；菜单栏图标由用户决定是否显示，且不能指望它一直在。打断研究（Iqbal & Bailey CHI 2008）表明：把提示推迟到任务断点再给，挫败感和反应时间都更低，平均只延后约 90 s。临床提示疲劳研究（Ancker 2017）显示：同一次诊疗里每多一次提醒，被接受的概率降 30%。
**草案**：默认只改菜单栏图标，不发系统通知；只在用户停止说话 ≥ 3 s 后才显示；同一条建议每场会话最多 1 次、每天最多 2 次；用户忽略两次就自动不再提；每条都能「不再提示」，并可一键全部关闭。整套提示是纯 UI，0 ms 音频延迟，算力可忽略。

---

## 证据表

| # | 产品/来源 | 发现 | 标记 | 出处 |
|---|---|---|---|---|
| 1 | Microsoft Teams | 静音状态下开口说话时，会议窗口内出现「Your microphone is muted」 | 【官方】（微软归档博客，非正式产品文档） | learn.microsoft.com/en-gb/archive/blogs/skypehybridguy/microsoft-teams-sorry-i-was-talking-on-mute（2017-11-17） |
| 2 | Teams | 能否单独关掉这条提示：**未查到**。只有一个全局开关，Settings › Notifications and activity › 取消勾选「Show notifications during calls and meetings」，且该开关不保证管得到这条 | 【官方】 | support.microsoft.com/teams/meetings/mute-notifications-during-a-meeting-in-microsoft-teams |
| 3 | Webex | 静音时说话会提示，40.11 版（2020-11）上线；**在本机分析麦克风采样，不上传云端** | 【官方】 | blog.webex.com/collaboration/video-conferencing/webex-audio-mute/ |
| 4 | Webex | 帮助页原意：静音时说话，会提示先取消静音或举手。阈值、频率限制均**未公开** | 【官方】 | help.webex.com/article/n94aj5j |
| 5 | Webex | 用户抱怨检测不灵（「得对着麦喊」），同时又有人想关掉这条提示 | 【传言】 | community.cisco.com/t5/…/talking-on-mute-notification-not-sensitive-enough/m-p/4290805 |
| 6 | Zoom | Workplace 6.2（2024-09）起：「静音时说话会持续给出提醒」 | 【传言】（TidBITS 转述发布说明） | tidbits.com/?p=68025 |
| 7 | Zoom | 「You are muted now」弹窗被打字、背景噪音、旁人说话误触发；在嘈杂环境里反复弹出，会抢焦点（按回车可能误取消静音），还会遮挡共享屏幕。从 2021-01 到 2022-10，用户反复要求加关闭开关，没有官方回复 | 【传言】（官方开发者论坛里的用户帖） | devforum.zoom.us/t/option-to-disable-you-are-muted-now-pop-up/41011 |
| 8 | Zoom | 「High background noise detected」之类的主动噪音提示：**未查到**。降噪只有设置项（Auto/Low/Medium/High） | 未查到 | — |
| 9 | Google Meet | 回声提示：「更多选项」按钮上出现红点并附文字；点开跳到帮助中心（建议调低音量、不说话时静音、戴耳机）；默认开启，**管理员无法控制** | 【官方】 | workspaceupdates.googleblog.com/2021/08/helping-users-prevent-echo-in-google.html（2021-08-23） |
| 10 | Google Meet | 声音指示圈按大小显示「正在去掉多少噪音」；其中「一次性提示」**每场会议最多出现一次**；不发其他通知 | 【官方】 | workspaceupdates.googleblog.com/2022/08/see-how-much-noise-is-being-removed-during-google-meet-calls.html（2022-08-16） |
| 11 | Apple HIG · Notifications | 「Avoid sending multiple notifications for the same thing, even if someone hasn't responded… people may turn off all notifications from your app.」以及「Use an alert — not a notification — to display an error message.」 | 【官方】 | developer.apple.com/design/human-interface-guidelines/notifications |
| 12 | Apple HIG · Alerts | 「Use alerts sparingly」「Avoid using an alert merely to provide information… prefer finding an alternative way to communicate it within the relevant context (e.g. Mail 的离线指示器)」；macOS 上「Configure repeating alerts to let people suppress subsequent occurrences」（即 suppression checkbox）；文案要「direct, neutral, approachable… avoid being oblique or accusatory」 | 【官方】 | developer.apple.com/design/human-interface-guidelines/alerts |
| 13 | Apple HIG · Menu bar | 「Let people — not your app — decide whether to put your menu bar extra in the menu bar.」以及「Avoid relying on the presence of menu bar extras… The system hides and shows menu bar extras regularly」；图标用模板符号（SF Symbols） | 【官方】 | developer.apple.com/design/human-interface-guidelines/the-menu-bar |
| 14 | Apple UNNotificationInterruptionLevel | `.passive`：只进通知列表，不亮屏、不出声；`.active`：立即呈现，可以出声；`.timeSensitive` 能穿透专注模式。macOS 12+ 可用 | 【官方】 | developer.apple.com/documentation/usernotifications/unnotificationinterruptionlevel |
| 15 | macOS 通知设置 | 有「Allow notifications when mirroring or sharing the display」开关，控制共享屏幕时是否显示通知（默认值本次未核实） | 【官方】 | support.apple.com/guide/mac-help/mchl205da693 |
| 16 | Phonak 助听器 | **切换进** Automatic 程序时播放提示音；AutoSense 在 Automatic 内部换声学场景时是否出声，官方页**没写** | 【官方】 | phonak.com/en-au/support/knowledge-base/articles/what-do-the-notification-sounds-i-hear-in-my-hearing-aid-mean |
| 17 | 助听器（Phonak/Oticon） | 用户报告：自动切换默认不响，手动切换才响；验配师可在验配软件里关掉提示音（Phonak Target › Device Options › Signals and Alerts；Oticon Genie 2） | 【传言】（HearingTracker 论坛） | forum.hearingtracker.com/t/phonak-users-who-dont-use-autosense/77546 ；forum.hearingtracker.com/t/automatic-program-change-issue-with-phonak-audeo-paradise/62345 |
| 18 | Buechler 2001（Phonak Claro AutoSelect，n=22） | 67% 的人能经常察觉到切换；26% 觉得「太频繁或太少」；只有 60% 认为切换符合预期（作者归因于 10 s 的分类延迟和个体差异）；建议让用户微调切换灵敏度。文中**没提**提示音 | 【论文】 | hearingreview.com/…/how-good-are-automatic-program-selection-features（2001-09） |
| 19 | Iqbal & Bailey, CHI 2008 | 提示推迟到断点再给，比立即给挫败感低（Policy 主效应 F(3,109)=5.4，p<0.002）；对相关提示的反应时间 3.07 s，立即给为 4.08 s；平均延后 88.6 s（SD 139.3），作者认为「< 90 s 是可接受的平衡」；粗断点（换任务）适合一般信息 | 【论文】 | Iqbal & Bailey CHI'08，p.99–101，Fig.3 |
| 20 | Adamczyk & Bailey, CHI 2004 | 在子任务边界打断，比在任务中途打断更省成本，对情绪的伤害更小 | 【论文】（摘要及他人引述，本次未逐页读全文） | CHI 2004 pp.271–278 |
| 21 | Ancker et al. 2017 | 同一次诊疗中每多收到 1 条提醒，被接受的概率降 30%；重复提醒占比每升 5 个百分点，接受率降 10%；不是工作负荷本身造成的 | 【论文】 | BMC Med Inform Decis Mak 17:36（doi 10.1186/s12911-017-0430-8），摘要 |

---

## 对本项目的含义

**落点**：提示不在音频回调里，算法延迟和缓冲**增加 0 ms**。判断沿用场景层已有的 0.5 s 判断回路（参数层），提示只读取场景层的状态，比如「多人 babble 持续时间」「近场主讲人能量比」「是否在说话（VAD）」，再写一个 UI 事件。算力是每 0.5 s 一次计数器比较，可以忽略。许可方面只用系统 API（NSStatusItem、UNUserNotificationCenter），与 GPL-3.0 没有冲突。不上云：同 Webex 的做法，所有判断都在本机完成。

**分层**：
- M1（只有 CLI 加虚拟麦克风）：不弹任何东西，只在 CLI 的状态输出（比如 `vdn status`）里写一行建议。
- M2（菜单栏 app）：启用下面的规则。
- 场景层：自动切换内部细场景时**不提示**。参照助听器：AutoSense 内部换场景不响（推断），只有进入用户能看到的程序才响。用户层的基本场景（比如「多人模式」）只**建议**，不自动切，因为自动切换会改变用户听到的效果，而且 Buechler 2001 中有 26% 的人嫌切换频率不对。

### 提示规则草案（v0，待人拍板）

| 维度 | 规则 | 依据 |
|---|---|---|
| **提示什么** | 只有两条建议：A「靠近麦克风或改用耳机麦克风效果更好」；B「周围多人说话，可以试试『多人模式』」。不做「你那边噪音大」的指责式提示 | #12 语气中性、不指责；#8 主流产品都没做噪音告警 |
| **何时触发** | 场景层连续判定「多人 babble 或旁人说话」累计 ≥ 60 s，近场主讲人能量比低于阈值，且当前不在多人模式（A 还要求当前输入设备是内置麦克风） | 推断：避免短时噪声误触发（参照 #7 Zoom 被噪声误触发） |
| **何时显示** | **只在用户停顿时显示**：本机 VAD 判定用户已停止说话 ≥ 3 s；如果 10 min 内一直等不到停顿，就放弃这次，不强行插入 | #19 断点推迟、平均延后 < 90 s 可接受；#20 |
| **在哪显示（默认）** | **只改菜单栏图标**：叠加一个小圆点或换成模板符号的变体（不闪烁、不出声），点开菜单后第一行是建议和一键操作（「切到多人模式」／「不再提示」） | #12 只告知信息时用上下文指示器，不用 alert；#9 Meet 红点模式 |
| **系统通知** | 默认关闭。用户在设置里打开后，才用 `.passive` 级别（不出声、不亮屏，只进通知中心），**绝不用** `.timeSensitive` 或 `.critical`；在共享屏幕时不发（遵从系统的镜像/共享开关，同时自查屏幕录制状态，属推断） | #11 #14 #15；#7 遮挡共享屏幕的教训 |
| **频率上限** | 同一条建议**每次会话（虚拟麦克风从被占用到释放）最多 1 次**；每天最多 2 次；两次之间至少隔 30 min | #10 Meet 每场一次；#11 不重复；#21 每多一次接受率降 30% |
| **忽略即退避** | 同一条建议连续被忽略 2 次（圆点出现后没点开，会话就结束了），就停 7 天；停过以后再被忽略，永久不再提，直到用户手动重置 | #21 重复提示会降低接受率；#11 否则用户会关掉全部通知 |
| **永久关闭** | 每条建议都有「不再提示这条」（对应 macOS 的 suppression checkbox）；设置里还有总开关「建议提示：开/关」；菜单栏图标是否显示也由用户决定 | #12 #13 |
| **措辞** | 第二人称、可选项、不带感叹号。例：「周围说话的人较多。试试多人模式？」「靠近麦克风或改用耳机麦克风，声音会更清楚。」不写「检测到噪音过大！」 | #12 direct、neutral、不指责 |
| **不出声** | 提示永远不能发声，因为我们在用户的通话链路里；也不往用户的输出设备里插提示音 | 推断：与助听器不同，我们的用户正在开会 |
| **不依赖菜单栏** | 菜单栏图标可能被隐藏，所以建议同时写进 app 主窗口或设置页的「最近建议」列表，不因此改发通知 | #13 |
| **静音时说话、回声** | 不做，这是会议软件的职责，我们看不到它的静音状态 | 推断 |

## gap（需实测或未查到）

1. **Zoom、Teams、Webex 的触发阈值、最短持续时间和频率限制**：官方都未公开，未查到。如需对齐，只能实测：静音后用不同 dB、不同时长说话，记录多久后出提示、是否会重复出。
2. **阈值标定**：「累计 60 s」「停顿 3 s」「隔 30 min」这几个数是推断值。需要在 exp003 的场景片段上量误触发率，并由人试用一周、回填打扰感（7 级量表，参照 #19）。
3. **VAD 停顿检测**：本项目在嘈杂多人场景中能否可靠区分「用户自己停顿」和「旁人在说」，取决于近场与远场的判别能力（与 PSE 线耦合），需要实测。
4. **共享屏幕检测**：app 能否可靠得知「正在共享屏幕」，以及 macOS 镜像/共享开关的默认值，本次未核实。
5. **菜单栏圆点能否被注意到**：圆点不出声、不闪，用户可能根本看不到。这是有意的取舍，被注意到的比例需要由人在实际使用中判断。
6. **Teams 能否单独关闭「静音时说话」提示**：未查到。
7. **Phonak AutoSense 在内部换场景时是否出声**：官方未写，只有论坛的相互矛盾的说法。
8. **Adamczyk & Bailey 2004 的具体数值**：本次未读全文。

## 来源清单

- https://learn.microsoft.com/en-gb/archive/blogs/skypehybridguy/microsoft-teams-sorry-i-was-talking-on-mute
- https://support.microsoft.com/teams/meetings/mute-notifications-during-a-meeting-in-microsoft-teams
- https://blog.webex.com/collaboration/video-conferencing/webex-audio-mute/
- https://help.webex.com/article/n94aj5j
- https://community.cisco.com/t5/webex-meetings-and-webex-app/talking-on-mute-notification-not-sensitive-enough/m-p/4290805
- https://tidbits.com/?p=68025
- https://devforum.zoom.us/t/option-to-disable-you-are-muted-now-pop-up/41011
- https://workspaceupdates.googleblog.com/2021/08/helping-users-prevent-echo-in-google.html
- https://workspaceupdates.googleblog.com/2022/08/see-how-much-noise-is-being-removed-during-google-meet-calls.html
- https://developer.apple.com/design/human-interface-guidelines/notifications
- https://developer.apple.com/design/human-interface-guidelines/alerts
- https://developer.apple.com/design/human-interface-guidelines/the-menu-bar
- https://developer.apple.com/documentation/usernotifications/unnotificationinterruptionlevel
- https://support.apple.com/guide/mac-help/mchl205da693
- https://www.phonak.com/en-au/support/knowledge-base/articles/what-do-the-notification-sounds-i-hear-in-my-hearing-aid-mean
- https://forum.hearingtracker.com/t/phonak-users-who-dont-use-autosense/77546
- https://hearingreview.com/hearing-loss/patient-care/evaluation/how-good-are-automatic-program-selection-features
- https://www.interruptions.net/literature/Iqbal-CHI08.pdf （Iqbal & Bailey, CHI 2008, pp.93–102）
- https://hci.rwth-aachen.de/materials/conferences/CHI2004/1p271.pdf （Adamczyk & Bailey, CHI 2004）
- https://bmcmedinformdecismak.biomedcentral.com/articles/10.1186/s12911-017-0430-8 （Ancker et al. 2017）
