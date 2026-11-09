"""Crop a gate reference (ROI-sized PNG) from a runner shot and print its sha256.

Usage: crop_ref.py <shot.png> <out.png> <x0> <y0> <x1> <y1>
"""
import hashlib
import sys

from PIL import Image


def crop_ref(shot, out, box):
    with Image.open(shot) as image:
        image.convert('RGB').crop(tuple(box)).save(out)
    return hashlib.sha256(open(out, 'rb').read()).hexdigest()


if __name__ == '__main__':
    print(crop_ref(sys.argv[1], sys.argv[2], [int(v) for v in sys.argv[3:7]]))
