"""coremltools 9.0 × numpy ≥ 2.5 的兼容补丁（只在本实验的转换脚本里用，不改 venv 里的库）。

症状：torch 前端 `_cast` 里 `int(x.val)` 遇到形状 (1,) 的数组，numpy 2.5 起直接报
「only 0-dimensional arrays can be converted to Python scalars」。做法：把单元素数组先压成标量再转。
"""
import numpy as np
from coremltools.converters.mil.frontend.torch import ops as _ops

_orig_cast = _ops._cast


def _scalar(dtype):
    def f(v):
        if isinstance(v, np.ndarray) and v.ndim > 0 and v.size == 1:
            v = v.reshape(())
        return dtype(v)
    return f


def _cast(context, node, dtype, dtype_name):
    return _orig_cast(context, node, _scalar(dtype), dtype_name)


_ops._cast = _cast
