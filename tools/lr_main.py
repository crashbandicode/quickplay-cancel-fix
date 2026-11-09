"""Resolve x30/lr values of a trace's records to main offsets, using the main base the trace's own pc/location pairs
give (Eden records `registers.pc` and `locations` "main+0x…" for each point).

Usage: lr_main.py <trace.jsonl> [kind ...]   (kinds as tools/timeline.py names them; default local_state)
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from timeline import kind_of, num  # noqa: E402


def main_base(records):
    for r in records:
        pc = num((r.get('registers') or {}).get('pc'))
        for loc in (r.get('locations') or {}).values():
            if pc is not None and isinstance(loc, str) and loc.startswith('main+0x'):
                return pc - int(loc[5:], 16)
    return None


def main(argv=None):
    argv = argv if argv is not None else sys.argv[1:]
    records = []
    for line in open(argv[0], encoding='utf-8', errors='replace'):
        try:
            records.append(json.loads(line))
        except ValueError:
            continue
    kinds = set(argv[1:]) or {'local_state'}
    base = main_base(records)
    print(f'main base {base:#x}' if base is not None else 'main base unknown')
    for r in records:
        kind = kind_of(r)
        if kind not in kinds:
            continue
        regs = r.get('registers') or {}
        lr = num(regs.get('x30'))
        off = f'main+{lr - base:#x}' if lr is not None and base is not None else '?'
        frames = [num(f.get('pc') if isinstance(f, dict) else f) for f in (r.get('backtrace') or [])]
        bt = ' '.join(f'{f - base:#x}' for f in frames if f is not None and base is not None)[:200]
        print(f'{num(r.get("frame"))} {kind} x1={regs.get("x1")} lr={off} bt=[{bt}]')
    return 0


if __name__ == '__main__':
    sys.exit(main())
