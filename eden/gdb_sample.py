"""Freeze A/B (primary 17:59Z): sample a frozen b's guest through Eden's GDB stub, via the `gdbdump.py hold` helper
already attached (steps.py:gdb_on): it answers <run>/out/gdb/dump-<n>.req (lines `hexaddr:len`) with
dump-<n>.json (every guest thread's x0-x30/sp/pc and the requested memory). Takes 4 register samples 1 s apart,
then walks the frame-pointer chain (saved x29/x30 pairs) of the threads whose PC stayed within 0x800 across the
samples, up to 16 frames, one dump per depth. Prints every thread's PC/LR per sample and the backtraces as main+offset
(the main module's base from the control socket's modules.list), and writes <dest>/gdb-samples.json.

Usage: gdb_sample.py <run dir> <dest dir>
"""
import json
import subprocess
import sys
import time
from pathlib import Path

CTL = ('/home/intpa/Applications/Eden-GIF/20261110-656ca2fcd596-unmapped-stop-lan2c-rmcdump-52288a74e874/tools/'
       'edenctl.py')
run, dest = Path(sys.argv[1]), Path(sys.argv[2])
folder = run / 'out' / 'gdb'
n = len(list(folder.glob('dump-*.json'))) + 1


def dump(ranges, cap=60.0):
    global n
    req, out = folder / f'dump-{n}.req', folder / f'dump-{n}.json'
    req.write_text(''.join(f'{r}\n' for r in ranges))
    end = time.monotonic() + cap
    while time.monotonic() < end:
        if out.exists():
            time.sleep(0.3)
            try:
                rec = json.loads(out.read_text())
                n += 1
                return rec
            except json.JSONDecodeError:
                pass
        time.sleep(0.2)
    raise SystemExit(f'no answer to {req} in {cap} s (is the hold helper attached? {folder / "hold.log"})')


def modules():
    try:
        out = subprocess.run(['/usr/bin/timeout', '30', '/usr/bin/python3', '-I', CTL, '--socket',
                              str(run / 'state' / 'run-b' / 'control.sock'), 'call', 'modules.list', '{}'],
                             capture_output=True, text=True, check=False).stdout
        return (json.loads(out).get('result') or {}).get('modules') or []
    except (ValueError, OSError):
        return []


mods = modules()
spans = []
for m in mods:
    try:
        spans.append((m.get('name') or m.get('path') or '?', int(str(m['base']), 16), int(str(m.get('size', 0)), 0)))
    except (KeyError, ValueError):
        pass


def where(v):
    try:
        a = int(v, 16)
    except (TypeError, ValueError):
        return str(v)
    for name, base, size in spans:
        if base <= a < base + size:
            return f'{name}+{a - base:#x}'
    return hex(a)


samples = []
for i in range(4):
    samples.append(dump([]))
    time.sleep(1.0)
threads = {}
for s in samples:
    for t in s.get('threads') or []:
        threads.setdefault(t['id'], {'name': t.get('name'), 'regs': []})['regs'].append(t.get('regs') or {})
spinning = []
print('modules:', [(m, hex(b), hex(sz)) for m, b, sz in spans][:12])
for tid, t in threads.items():
    pcs = [int(r.get('pc', '0x0'), 16) for r in t['regs'] if r.get('pc')]
    print(f"thread {tid} {t['name']}: pc " + ' '.join(where(r.get('pc')) for r in t['regs'])
          + ' | lr ' + ' '.join(where(r.get('x30')) for r in t['regs']))
    if pcs and max(pcs) - min(pcs) < 0x800 and len(set(pcs)) > 1:
        spinning.append(tid)
# Threads whose PC moved within a small window across the samples are the spinning ones; if none moved, walk all
# threads on a core whose PC is in main.
chains = {tid: [] for tid in (spinning or list(threads))}
fps = {tid: threads[tid]['regs'][-1].get('x29') for tid in chains}
for depth in range(16):
    ranges = {tid: f'{int(fp, 16):x}:16' for tid, fp in fps.items() if fp and int(fp, 16) > 0x1000}
    if not ranges:
        break
    rec = dump(sorted(set(ranges.values())))
    mem = rec.get('memory') or {}
    nxt = {}
    for tid, r in ranges.items():
        raw = mem.get(r) or ''
        if len(raw) < 32 or raw.startswith('E'):
            continue
        b = bytes.fromhex(raw[:32])
        fp, lr = int.from_bytes(b[:8], 'little'), int.from_bytes(b[8:], 'little')
        chains[tid].append(hex(lr))
        if fp:
            nxt[tid] = hex(fp)
    fps = nxt
print('spinning (PC moved within 0x800):', spinning)
for tid, chain in chains.items():
    print(f"backtrace {tid} {threads[tid]['name']}: pc {where(threads[tid]['regs'][-1].get('pc'))} lr "
          f"{where(threads[tid]['regs'][-1].get('x30'))} <- " + ' <- '.join(where(a) for a in chain))
dest.mkdir(parents=True, exist_ok=True)
json.dump({'modules': mods, 'samples': samples, 'spinning': spinning, 'chains': chains},
          open(dest / 'gdb-samples.json', 'w'), indent=1)
