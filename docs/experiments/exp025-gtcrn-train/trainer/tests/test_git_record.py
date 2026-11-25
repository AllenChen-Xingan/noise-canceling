"""运行记录必须带 git 提交号：拿不到就在开跑前失败，不写空值（大总管 2026-11-25，起因：云上用 git archive 解包，
没有 .git，run.json 里 commit 为空、dirty 却记成 False，看着像「干净的提交」）。"""
import pytest

import runrec
import train
from test_train import _cfg


def test_git_info_raises_outside_repo(tmp_path, monkeypatch):
    monkeypatch.setattr(runrec, "HERE", tmp_path)  # tmp_path 不在任何 git 仓库里
    with pytest.raises(RuntimeError, match="git"):
        runrec.git_info()


def test_git_info_raises_without_git_binary(monkeypatch):
    def no_git(*a, **k):
        raise FileNotFoundError("git")
    monkeypatch.setattr(runrec.subprocess, "run", no_git)
    with pytest.raises(RuntimeError, match="git"):
        runrec.git_info()


def test_train_refuses_before_touching_run_dir(tmp_path, monkeypatch):
    """开跑前就失败：不建运行目录、不标定、不花钱。"""
    monkeypatch.setattr(runrec, "HERE", tmp_path / "not-a-repo")
    (tmp_path / "not-a-repo").mkdir()
    cfg = _cfg(tmp_path / "runs")
    with pytest.raises(RuntimeError, match="git"):
        train.train(cfg)
    assert not (tmp_path / "runs").exists()


def test_git_info_in_repo_has_commit():
    g = runrec.git_info()
    assert len(g["commit"]) == 40 and isinstance(g["dirty"], bool)
