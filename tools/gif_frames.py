"""Inspect and trim a raw Eden GIF take for the README (playbook GIFS.md): list frame times, write contact frames,
or cut a time range into a new GIF (same frames, same durations; 1280 wide kept).

Usage:
  gif_frames.py info <in.gif>
  gif_frames.py frames <in.gif> <out dir> <t0> <t1> [<step s>]      PNGs of the frames shown at t0, t0+step, ... (s)
  gif_frames.py cut <in.gif> <out.gif> <t0> <t1>                     the frames shown between t0 and t1 (s)
  gif_frames.py splice <out.gif> <in.gif> <t0> <t1> [<in.gif> <t0> <t1> ...]   pieces of takes, in order, as one GIF
"""
import sys
from pathlib import Path

from PIL import Image


def timeline(path):
    """[(start s, duration s)] per frame."""
    out, t = [], 0.0
    with Image.open(path) as image:
        for i in range(getattr(image, 'n_frames', 1)):
            image.seek(i)
            d = image.info.get('duration', 0) / 1000
            out.append((t, d))
            t += d
    return out


def frame_at(times, t):
    for i, (start, d) in enumerate(times):
        if start <= t < start + d:
            return i
    return len(times) - 1


def cut(src, dst, t0, t1):
    times = timeline(src)
    keep = [i for i, (start, d) in enumerate(times) if start + d > t0 and start < t1]
    frames, durations = [], []
    with Image.open(src) as image:
        for i in keep:
            image.seek(i)
            frames.append(image.convert('RGB'))
            start, d = times[i]
            shown = min(start + d, t1) - max(start, t0)
            durations.append(max(20, int(round(shown * 1000))))
    frames[0].save(dst, save_all=True, append_images=frames[1:], duration=durations, loop=0, optimize=True)
    return len(frames), sum(durations) / 1000


def splice(dst, parts):
    """One GIF from (src, t0, t1) pieces in order, each cut as `cut` does; returns (frames, seconds)."""
    frames, durations = [], []
    for src, t0, t1 in parts:
        times = timeline(src)
        with Image.open(src) as image:
            for i, (start, d) in enumerate(times):
                if start + d <= t0 or start >= t1:
                    continue
                image.seek(i)
                frames.append(image.convert('RGB'))
                durations.append(max(20, int(round((min(start + d, t1) - max(start, t0)) * 1000))))
    frames[0].save(dst, save_all=True, append_images=frames[1:], duration=durations, loop=0, optimize=True)
    return len(frames), sum(durations) / 1000


def main(argv):
    mode = argv[0]
    if mode == 'info':
        times = timeline(argv[1])
        with Image.open(argv[1]) as image:
            size = image.size
        total = times[-1][0] + times[-1][1] if times else 0
        print(f'{argv[1]}: {size[0]}x{size[1]}, {len(times)} frames, {total:.2f} s')
        return 0
    if mode == 'frames':
        src, out, t0, t1 = argv[1], Path(argv[2]), float(argv[3]), float(argv[4])
        step = float(argv[5]) if len(argv) > 5 else 1.0
        out.mkdir(parents=True, exist_ok=True)
        times = timeline(src)
        with Image.open(src) as image:
            t = t0
            while t <= t1:
                i = frame_at(times, t)
                image.seek(i)
                image.convert('RGB').save(out / f't{t:06.2f}-f{i:04d}.png')
                t += step
        return 0
    if mode == 'cut':
        n, seconds = cut(argv[1], argv[2], float(argv[3]), float(argv[4]))
        print(f'{argv[2]}: {n} frames, {seconds:.2f} s')
        return 0
    if mode == 'splice':
        rest = argv[2:]
        parts = [(rest[k], float(rest[k + 1]), float(rest[k + 2])) for k in range(0, len(rest) - 2, 3)]
        n, seconds = splice(argv[1], parts)
        print(f'{argv[1]}: {n} frames, {seconds:.2f} s from {len(parts)} piece(s)')
        return 0
    print(__doc__)
    return 2


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
