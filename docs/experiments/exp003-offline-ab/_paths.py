"""exp003 各脚本共用的路径。

ROOT = 本工作副本的根（_tmp/ 与 docs/ 写这里）。
MAIN = 放 resources/ 的那份工作副本：resources/ 被 git 忽略，只在主目录里有；
       在 worktree 里跑时，按 git 的公共目录找回主目录，不在 worktree 里重新克隆。
"""
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _main_root():
    if (ROOT / "resources/weights").is_dir():  # worktree 里也有个只放 README 的 resources/，要看实物
        return ROOT
    common = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--path-format=absolute", "--git-common-dir"],
                            capture_output=True, text=True, check=True).stdout.strip()
    main = Path(common).parent
    if not (main / "resources/weights").is_dir():
        raise FileNotFoundError(f"找不到 resources/：{ROOT} 与 {main} 下都没有")
    return main


MAIN = _main_root()
RES = MAIN / "resources"
