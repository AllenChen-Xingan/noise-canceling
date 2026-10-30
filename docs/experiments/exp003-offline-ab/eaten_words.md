# exp003 · 吃字逐段明细（脚本生成，勿手改）

ASR：qwen3-asr-plus:1.7b，每个文件转写两次，逐字一致 48/48。
「新删」= 降噪后转写里被删、原声转写里没被删的参照词。参照 = 干净参考的转写。

## c02 · 赌场环境（人声 + 机器声），SNR 0 dB

- 参照（干净参考转写）：The stray cat gave birth to kittens. The lazy cow lay in the cool grass. The friendly gang left the drug store. We talked of the sideshow in the circus. The set of china hit the floor with a crash.
- 原声转写：The stray cat gave birth to kittens. The lazy cow lay in the cool grass. The friendly gang left the drugstore. We talked of the sideshow in the circus. The set of china hit the floor with a crash.（两次一致：True）
- **gtcrn** 新删 2 个：store、crash
  - 参照中位置（加粗）：the stray cat gave birth to kittens the lazy cow lay in the cool grass the friendly gang left the drug **store** we talked of the sideshow in the circus the set of china hit the floor with a **crash**
  - gtcrn 转写：The stray cat gave birth to kittens. The lazy cow lay in the cool grass. The friendly gang left the drug. We caught of the sideshow in the circus. The set of china hit the floor with a.（两次一致：True）

## c03 · 赌场环境（人声 + 机器声），SNR −5 dB

- 参照（干净参考转写）：Clams are small, round, soft, and tasty. The line where the edges join was clean. Stop whistling and watch the boys march. A cruise in warm waters in a sleek yacht is fun.
- 原声转写：Clams are small, round, soft, and tasty. Stop whistling and watch the boys march. Clams in warm waters in the salt water.（两次一致：True）
- **dpdfnet2** 新删 1 个：yacht
  - 参照中位置（加粗）：clams are small round soft and tasty the line where the edges join was clean stop whistling and watch the boys march a cruise in warm waters in a sleek **yacht** is fun
  - dpdfnet2 转写：Clams are small, round, soft, and tasty. Doc whistled and watched the boys work. He firmed in warm waters and a sleep yawn.（两次一致：True）
- **gtcrn** 新删 4 个：stop、whistling、and、watch
  - 参照中位置（加粗）：clams are small round soft and tasty the line where the edges join was clean **stop** **whistling** **and** **watch** the boys march a cruise in warm waters in a sleek yacht is fun
  - gtcrn 转写：Clams are small, then soft and tasty. That's mostly what's to be learned. I played in one boys' and schoolyard.（两次一致：True）

## c04 · 邻座一人说话 + 咖啡馆露台底噪，SNR 5 dB

- 人工转写（LibriSpeech）：by this time lord chelford and wylder returned and disgusted rather with myself i ruminated on my want of general ship
- 参照（干净参考转写）：By this time, Lord Chelford and Wilder returned, and disgusted rather with myself, I ruminated on my want of generalship.
- 原声转写：By this time, Lord Chelford and Wilder returned, and disgusted rather with myself, I ruminated on my want of generalship.（两次一致：True）
- **gtcrn** 新删 1 个：with
  - 参照中位置（加粗）：by this time lord chelford and wilder returned and disgusted rather **with** myself i ruminated on my want of generalship
  - gtcrn 转写：What I mean is that I, by this time, Lord Chelford and Wilder returned, and disgusted rather myself, I ruminated on my want of generalship.（两次一致：True）

## c06 · 邻桌 3 人说话 + 食堂底噪，SNR 0 dB

- 人工转写（LibriSpeech）：his troubled blue eyes glanced at each of us and finally rested with an expression of blank dismay upon bannister in the farther corner
- 参照（干净参考转写）：His troubled blue eyes glanced at each of us, and finally rested, with an expression of blank dismay, upon Bannister in the farther corner.
- 原声转写：His troubled blue eyes glanced at each of us, and finally rested, with an expression of blank dismay, upon Bannister room farther on. Your aunt Rachel.（两次一致：True）
- **dpdfnet2** 新删 3 个：bannister、in、the
  - 参照中位置（加粗）：his troubled blue eyes glanced at each of us and finally rested with an expression of blank dismay upon **bannister** **in** **the** farther corner
  - dpdfnet2 转写：His troubled blue eyes glanced at each of us, and finally rested with an expression of blank dismay upon Banister Brindle.（两次一致：True）

## c11 · 中文 · 邻桌 3 人（英文）说话 + 食堂底噪，SNR 0 dB

- 参照（干净参考转写）：红色的草莓口味与黄色的香蕉口味交替涂抹在甜筒上。近年来，销售员将冰品盛装成精致玫瑰花形的。
- 原声转写：红色的草莓口味与黄色的香蕉口味交替涂抹在甜筒上。多年来，销售员将冰品盛装成精致玫瑰花形。（两次一致：True）
- **dpdfnet2** 新删 3 个：红、色、的
  - 参照中位置（加粗）：**红** 色 的 草 莓 口 味 与 黄 **色** **的** 香 蕉 口 味 交 替 涂 抹 在 甜 筒 上 近 年 来 销 售 员 将 冰 品 盛 装 成 精 致 玫 瑰 花 形 的
  - dpdfnet2 转写：这个草莓口味与黄桃加草莓交替涂抹在皮肤上，接下来，销售员将产品盛装成一束玫瑰花形。（两次一致：True）

## 合计（新删词数）

| 候选 | 新删 |
|---|---|
| dpdfnet2 | 7 |
| gtcrn | 7 |
