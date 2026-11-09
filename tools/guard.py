"""Print the 64 guard bytes at each main-image offset of the 13.0.5 main ELF as a Rust array, for model.rs.

Usage: guard.py <main.elf> <hex offset> [...]
"""
import struct
import sys


def text_segment(data):
    phoff, = struct.unpack_from('<Q', data, 0x20)
    phnum, = struct.unpack_from('<H', data, 0x38)
    for i in range(phnum):
        p_type, p_flags, p_off, p_va, _, p_filesz, _, _ = struct.unpack_from('<IIQQQQQQ', data, phoff + i * 56)
        if p_type == 1 and p_flags & 1:
            return p_va, p_off, p_filesz
    raise SystemExit('no executable segment')


def main():
    data = open(sys.argv[1], 'rb').read()
    va0, off0, size = text_segment(data)
    for arg in sys.argv[2:]:
        offset = int(arg, 16)
        if not (va0 <= offset and offset + 64 <= va0 + size):
            raise SystemExit(f'{offset:#x} is outside .text')
        raw = data[off0 + offset - va0:off0 + offset - va0 + 64]
        rows = [', '.join(f'0x{b:02x}' for b in raw[i:i + 15]) for i in range(0, 64, 15)]
        print(f'// {offset:#x}')
        print('guard: [\n' + ''.join(f'    {row},\n' for row in rows) + '],')


if __name__ == '__main__':
    main()
