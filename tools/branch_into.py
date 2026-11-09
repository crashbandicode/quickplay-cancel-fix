"""List direct branches (b, bl, b.cond, cbz/cbnz, tbz/tbnz) anywhere in .text of the 13.0.5 main ELF whose target
falls inside [start, end): a hook that replaces those words must not have code jumping into its middle.

Usage: branch_into.py <main.elf> <hex start> <hex end>
"""
import array
import struct
import sys


def text_words(data):
    phoff, = struct.unpack_from('<Q', data, 0x20)
    phnum, = struct.unpack_from('<H', data, 0x38)
    for i in range(phnum):
        p_type, p_flags, p_off, p_va, _, p_filesz, _, _ = struct.unpack_from('<IIQQQQQQ', data, phoff + i * 56)
        if p_type == 1 and p_flags & 1:
            words = array.array('I')
            words.frombytes(data[p_off:p_off + p_filesz - p_filesz % 4])
            return p_va, words
    raise SystemExit('no executable segment')


def target(word, va):
    """The branch target of `word` at `va`, or None."""
    if word & 0x7c000000 == 0x14000000:  # b / bl
        return va + (((word & 0x03ffffff) ^ 0x02000000) - 0x02000000) * 4
    if word & 0xff000010 == 0x54000000:  # b.cond
        return va + ((((word >> 5) & 0x7ffff) ^ 0x40000) - 0x40000) * 4
    if word & 0x7e000000 == 0x34000000:  # cbz / cbnz
        return va + ((((word >> 5) & 0x7ffff) ^ 0x40000) - 0x40000) * 4
    if word & 0x7e000000 == 0x36000000:  # tbz / tbnz
        return va + ((((word >> 5) & 0x3fff) ^ 0x2000) - 0x2000) * 4
    return None


def main():
    data = open(sys.argv[1], 'rb').read()
    lo, hi = int(sys.argv[2], 16), int(sys.argv[3], 16)
    va0, words = text_words(data)
    found = 0
    for i, word in enumerate(words):
        va = va0 + 4 * i
        t = target(word, va)
        if t is not None and lo <= t < hi:
            print(f'{va:#x} {word:08x} -> {t:#x}')
            found += 1
    print(f'{found} branch(es) into [{lo:#x}, {hi:#x})')


if __name__ == '__main__':
    main()
