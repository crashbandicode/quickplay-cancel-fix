"""Check README GIFs against the playbook's rules: 1280 wide, under 10 s.

Usage: gifcheck.py <file.gif> [...]
"""
import sys

from PIL import Image

WIDTH = 1280
MAX_SECONDS = 10.0


def check_gif(path):
    """(ok, width, seconds, frames) for the GIF at `path`."""
    with Image.open(path) as image:
        frames = getattr(image, 'n_frames', 1)
        duration = 0
        for frame in range(frames):
            image.seek(frame)
            duration += image.info.get('duration', 0)
        width = image.width
    seconds = duration / 1000
    return width == WIDTH and seconds < MAX_SECONDS, width, seconds, frames


def main(argv=None):
    ok_all = True
    for path in argv if argv is not None else sys.argv[1:]:
        ok, width, seconds, frames = check_gif(path)
        ok_all &= ok
        print(f'{"PASS" if ok else "FAIL"} {path}: {width} px, {seconds:.2f} s, {frames} frames')
    return 0 if ok_all else 1


if __name__ == '__main__':
    sys.exit(main())
