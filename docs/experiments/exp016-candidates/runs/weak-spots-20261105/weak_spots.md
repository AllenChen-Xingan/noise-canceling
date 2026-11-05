# GTCRN 弱在哪（由 weak_spots.py 生成，勿手改）

来源 run：`20261105-005916-4ccf0638+20261105-103000-4cf04ac1` · `20261105-005916-4ccf0638+20261105-115707-fcf7cfb0`。候选：dfn3-ll · dpdfnet2。
弱项 = 每个候选都比 GTCRN 好（95% 区间不跨 0）、配对单位 ≥ 5；守住项 = 每个候选都比 GTCRN 差。不看 V-CLICK · V-LAT · V-NAN · V-VACUUM。差值正 = 候选更好。未做多重比较校正；全部是合成音频。

## 弱项：83 个「场景 × 声音」，395 个格子

| 场景 | 声音 | 格子数 | 评分项 |
|---|---|---|---|
| car-passenger | bed-car-mcv | 9 | V-EAT-ASR · V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-TSOS · V-WEAK |
| cowork | bed-ace | 8 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-TSOS · V-WEAK |
| cowork | talker | 8 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SIM-FA · V-SPLVL · V-WEAK |
| car-passenger | talker | 8 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-TSOS · V-WEAK |
| transit-hub | bed-ace | 8 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-TSOS · V-WEAK |
| transit-hub | luggage | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| transit-hub | talker | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| transit-hub | announce | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| transit-hub | pa-chime | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| cafe | music | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| street-outdoor | talker | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| cafe | bed-restaurant | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| cafe | talker | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| cowork | coffee-machine | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| home-family | bed-kitchen | 7 | V-ESTOI · V-INTF · V-NR · V-NRMOS-DISTILL · V-PSE-OVL · V-SIM-FA · V-WEAK |
| car-passenger | nav-voice | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PSE-OVL · V-SPLVL · V-WEAK |
| car-passenger | turn-signal | 7 | V-EAT-ASR · V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-SPLVL · V-WEAK |
| open-office | bed-ace | 7 | V-ESTOI · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-SIM-FA · V-SPLVL · V-WEAK |
| library-classroom | talker | 7 | V-ESTOI · V-MUSIC · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-TSOS · V-WEAK |
| hotel-room | door-knock | 7 | V-ESTOI · V-MUSIC · V-NR · V-NRMOS-DISTILL · V-NRMOS-SCOREQ · V-PUMPV · V-WEAK |

### 弱项：标准化效应最大的 20 个格子

| 场景 | 声音 | 评分项 | 段数（单位） | dfn3-ll | dpdfnet2 |
|---|---|---|---|---|---|
| street-outdoor | wind | V-NRMOS-DISTILL | 10（5） | +0.159 Δ MOS [+0.134, +0.186] | +0.223 Δ MOS [+0.19, +0.259] |
| street-outdoor | wind | V-ESTOI | 10（5） | +0.0333 Δ [+0.0271, +0.0409] | +0.0411 Δ [+0.0355, +0.0489] |
| street-outdoor | bed-terrace | V-NRMOS-DISTILL | 10（5） | +0.15 Δ MOS [+0.111, +0.187] | +0.222 Δ MOS [+0.183, +0.258] |
| transit-hub | bed-ace | V-ESTOI | 10（5） | +0.0311 Δ [+0.0226, +0.0383] | +0.0486 Δ [+0.0433, +0.053] |
| street-outdoor | footsteps | V-ESTOI | 10（5） | +0.0329 Δ [+0.0312, +0.0347] | +0.0375 Δ [+0.0354, +0.0402] |
| street-outdoor | car-horn | V-ESTOI | 20（10） | +0.0296 Δ [+0.0243, +0.0351] | +0.0365 Δ [+0.0317, +0.0419] |
| transit-hub | bed-ace | V-NRMOS-DISTILL | 10（5） | +0.135 Δ MOS [+0.106, +0.166] | +0.193 Δ MOS [+0.161, +0.24] |
| cafe | music | V-ESTOI | 18（9） | +0.0284 Δ [+0.0219, +0.0358] | +0.0418 Δ [+0.0355, +0.0486] |
| street-outdoor | car-horn | V-NRMOS-DISTILL | 20（10） | +0.13 Δ MOS [+0.1, +0.159] | +0.197 Δ MOS [+0.166, +0.226] |
| street-outdoor | talker | V-ESTOI | 10（5） | +0.0281 Δ [+0.0209, +0.0331] | +0.0347 Δ [+0.0287, +0.0397] |
| transit-hub | luggage | V-ESTOI | 18（9） | +0.0279 Δ [+0.0231, +0.0323] | +0.0378 Δ [+0.0284, +0.0461] |
| cafe | music | V-NRMOS-SCOREQ | 18（9） | +0.156 Δ MOS [+0.106, +0.203] | +0.198 Δ MOS [+0.129, +0.272] |
| cafe | coffee-machine | V-ESTOI | 16（8） | +0.0274 Δ [+0.0167, +0.0377] | +0.0392 Δ [+0.029, +0.0492] |
| cowork | bed-ace | V-PSE-OVL | 12（6） | +1.87 dB [+0.789, +3.02] | +2.96 dB [+1.67, +4.24] |
| street-outdoor | footsteps | V-NRMOS-DISTILL | 10（5） | +0.125 Δ MOS [+0.104, +0.142] | +0.191 Δ MOS [+0.168, +0.212] |
| street-outdoor | bed-street | V-ESTOI | 22（11） | +0.0267 Δ [+0.0214, +0.0311] | +0.0331 Δ [+0.0285, +0.0372] |
| transit-hub | announce | V-ESTOI | 18（9） | +0.0263 Δ [+0.0199, +0.0327] | +0.0387 Δ [+0.0298, +0.0468] |
| transit-hub | pa-chime | V-ESTOI | 18（9） | +0.0263 Δ [+0.0199, +0.0327] | +0.0387 Δ [+0.0299, +0.0466] |
| canteen | bed-restaurant | V-NRMOS-DISTILL | 20（10） | +0.122 Δ MOS [+0.0874, +0.158] | +0.179 Δ MOS [+0.121, +0.242] |
| cowork | footsteps | V-ESTOI | 18（9） | +0.0262 Δ [+0.019, +0.035] | +0.0392 Δ [+0.0322, +0.0467] |

## 守住项（加训时别退）：71 个「场景 × 声音」，136 个格子

| 场景 | 声音 | 格子数 | 评分项 |
|---|---|---|---|
| rail-air-cabin | bed-cabin | 4 | V-MUSIC · V-ONSET · V-SIM-GATE · V-TSOS |
| street-outdoor | bed-terrace | 4 | V-BGSTAB · V-ONSET · V-SIM-GATE · V-TSOS |
| transit-hub | bed-station | 4 | V-BGSTAB · V-ONSET · V-TIMBRE · V-TSOS |
| street-outdoor | car-horn | 4 | V-BGSTAB · V-ONSET · V-SIM-GATE · V-TSOS |
| bus-metro-cabin | talker | 3 | V-BGSTAB · V-TIMBRE · V-TSOS |
| rail-air-cabin | talker | 3 | V-ONSET · V-SIM-GATE · V-TSOS |
| bus-metro-cabin | announce | 3 | V-BGSTAB · V-ONSET · V-TSOS |
| bus-metro-cabin | pa-chime | 3 | V-BGSTAB · V-ONSET · V-TSOS |
| transit-hub | babble | 3 | V-BGSTAB · V-ONSET · V-TIMBRE |
| transit-hub | footsteps | 3 | V-ONSET · V-SIM-GATE · V-TIMBRE |
| cafe | dishes | 3 | V-ONSET · V-SIM-GATE · V-TSOS |
| open-office | talker | 3 | V-ONSET · V-SIM-GATE · V-SWALLOW |
| street-outdoor | wind | 3 | V-BGSTAB · V-ONSET · V-SIM-GATE |
| home-family | bed-kitchen | 3 | V-BGSTAB · V-ONSET · V-TIMBRE |
| hotel-room | bed-ace | 3 | V-ONSET · V-SIM-GATE · V-TIMBRE |
| street-outdoor | bed-street | 3 | V-ONSET · V-SIM-GATE · V-TSOS |
| bus-metro-cabin | bed-bus | 2 | V-BGSTAB · V-ONSET |
| canteen | babble | 2 | V-BGSTAB · V-ONSET |
| cowork | footsteps | 2 | V-BGSTAB · V-ONSET |
| bus-metro-cabin | door-pneumatic | 2 | V-BGSTAB · V-TIMBRE |

### 守住项（加训时别退）：标准化效应最大的 20 个格子

| 场景 | 声音 | 评分项 | 段数（单位） | dfn3-ll | dpdfnet2 |
|---|---|---|---|---|---|
| rail-air-cabin | announce | V-ONSET | 14（7） | -2.13 dB [-3.16, -1.28] | -2.88 dB [-4.06, -1.56] |
| rail-air-cabin | pa-chime | V-ONSET | 14（7） | -2.13 dB [-3.16, -1.26] | -2.88 dB [-4.01, -1.53] |
| bus-metro-cabin | door-pneumatic | V-TIMBRE | 12（6） | -0.735 dB [-0.961, -0.496] | -0.648 dB [-1.29, -0.135] |
| rail-air-cabin | trolley | V-ONSET | 16（8） | -1.92 dB [-2.97, -1.05] | -2.64 dB [-3.74, -1.46] |
| street-outdoor | bed-terrace | V-ONSET | 10（5） | -1.9 dB [-3.22, -0.709] | -2.54 dB [-3.66, -1.33] |
| transit-hub | footsteps | V-ONSET | 14（7） | -2.09 dB [-2.59, -1.55] | -2.24 dB [-2.91, -1.53] |
| rail-air-cabin | bed-cabin | V-ONSET | 32（16） | -1.77 dB [-2.4, -1.16] | -2.51 dB [-3.24, -1.76] |
| cowork | talker | V-ONSET | 18（9） | -1.92 dB [-2.59, -1.27] | -2.2 dB [-2.97, -1.39] |
| canteen | bed-cafeteria | V-ONSET | 12（6） | -1.71 dB [-2.16, -1.21] | -3.28 dB [-3.82, -2.66] |
| rail-air-cabin | talker | V-ONSET | 30（15） | -1.7 dB [-2.38, -1.07] | -2.45 dB [-3.23, -1.62] |
| street-outdoor | wind | V-ONSET | 10（5） | -1.9 dB [-3.32, -0.576] | -2.05 dB [-3.46, -0.654] |
| canteen | dishes | V-ONSET | 26（13） | -1.64 dB [-2.05, -1.24] | -3.31 dB [-4.16, -2.67] |
| canteen | bed-restaurant | V-ONSET | 17（9） | -1.63 dB [-2.22, -1.05] | -3.03 dB [-4.35, -1.81] |
| canteen | talker | V-ONSET | 21（11） | -1.59 dB [-2.12, -1.09] | -3.03 dB [-4.13, -2.06] |
| cowork | footsteps | V-ONSET | 12（6） | -1.57 dB [-2.85, -0.0884] | -2.15 dB [-3.14, -0.877] |
| bus-metro-cabin | babble | V-TIMBRE | 10（5） | -0.625 dB [-0.9, -0.361] | -0.486 dB [-1.37, -0.0137] |
| canteen | babble | V-ONSET | 24（12） | -1.54 dB [-1.95, -1.14] | -3.1 dB [-4.07, -2.21] |
| bus-metro-cabin | brake | V-ONSET | 11（6） | -1.55 dB [-2.7, -0.888] | -1.79 dB [-1.9, -1.59] |
| transit-hub | babble | V-ONSET | 18（9） | -1.45 dB [-2.08, -0.81] | -1.78 dB [-2.64, -0.846] |
| cafe | dishes | V-ONSET | 19（10） | -1.45 dB [-2.07, -0.879] | -1.78 dB [-2.89, -0.736] |
