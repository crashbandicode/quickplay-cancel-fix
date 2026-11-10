"""Mean absolute difference per colour byte of a region in two frames (the runner's roi_diff, full-frame reference),
or with --roi-ref a reference that already is the region.

Usage: roi_check.py <frame.png> <ref.png> <x0> <y0> <x1> <y1> [--roi-ref]
"""
import sys

from PIL import Image, ImageChops, ImageStat


def roi_diff(frame, ref, box, ref_is_roi=False):
    with Image.open(frame) as a, Image.open(ref) as b:
        crop = a.convert('RGB').crop(tuple(box))
        other = b.convert('RGB') if ref_is_roi else b.convert('RGB').crop(tuple(box))
        if other.size != crop.size:
            other = other.resize(crop.size)
        return sum(ImageStat.Stat(ImageChops.difference(crop, other)).mean) / 3


if __name__ == '__main__':
    print(round(roi_diff(sys.argv[1], sys.argv[2], [int(v) for v in sys.argv[3:7]], '--roi-ref' in sys.argv), 2))
