"""exp008 第二轮 · mossformer_gan（ClearerVoice，非因果）。用法：python enh/mossformer_gan.py <build> [--jobs N] [--limit N] [--cells S-SNR,...]
跑法与记录格式见 enh/README.md；共用实现在 enh/clearvoice_common.py。"""
from clearvoice_common import main

if __name__ == "__main__":
    main("mossformer_gan")
