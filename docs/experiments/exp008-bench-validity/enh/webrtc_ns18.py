"""exp008 enh：webrtc_ns18（驱动见 _rustdsp.py）。用法：python enh/webrtc_ns18.py <build> [--jobs N] [--limit N]"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _rustdsp import main  # noqa: E402

sys.exit(main("webrtc_ns18"))
