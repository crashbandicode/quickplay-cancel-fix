"""Cut [start, end) seconds out of a GIF, keeping each frame's own duration and the size (no re-encode of pixels beyond
Pillow's palette save). Prints the frame count, duration and size of the result.

Usage: gif_trim.py <in.gif> <out.gif> <start_s> <end_s>
"""
import sys

from PIL import Image, ImageSequence

src, dst, start, end = sys.argv[1], sys.argv[2], float(sys.argv[3]), float(sys.argv[4])
frames, durations, t = [], [], 0.0
with Image.open(src) as im:
    for frame in ImageSequence.Iterator(im):
        d = frame.info.get('duration', 100) / 1000.0
        if t + d > start and t < end:
            frames.append(frame.convert('RGB'))
            durations.append(int(round((min(t + d, end) - max(t, start)) * 1000)))
        t += d
frames[0].save(dst, save_all=True, append_images=frames[1:], duration=durations, loop=0, optimize=True)
print(f'{dst}: {len(frames)} frames, {sum(durations) / 1000:.2f} s, {frames[0].size}')
