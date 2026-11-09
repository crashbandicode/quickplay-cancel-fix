"""Verdicts for a QCF Eden run from each instance's trace stream and desync-detector log.

Usage: verdict.py --trace a=<trace.jsonl> --trace b=<trace.jsonl> [--detector a=<events.log> ...] [--json]

- round starts: every host round start (`.1566af4`) with the path that started it, `countdown` (the room
  countdown had run out) or `full_room` (the room was full for 180 frames), from the waiting battle's
  full-room counter captured there;
- leave attempts: every PracticeScene leave (state 1 -> 4) and its outcome, `accepted` (4 -> anything but 1)
  or `refused` (4 -> 1, a battle round had started), with the round ack mask captured at the leave state;
- desync: the detector's `sync_end` lines, clean when each says `desync_seen=0`.
"""
import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from timeline import cap, kind_of, num  # noqa: E402

FULL_ROOM_WAIT = 180


def records(lines):
    for line in lines:
        try:
            yield json.loads(line)
        except ValueError:
            continue


def round_start_paths(trace):
    """[(frame, path, counter)] for each host round start in the trace records."""
    out = []
    for r in trace:
        if kind_of(r) != 'host_round_start':
            continue
        counter = num(cap(r, 0))
        if counter is None:
            path = 'unknown'
        else:
            path = 'full_room' if counter >= FULL_ROOM_WAIT else 'countdown'
        out.append((num(r.get('frame')) or 0, path, counter))
    return out


def leave_outcomes(trace):
    """[(frame, outcome, mask)] for each PracticeScene leave: state 1 -> 4, then 4 -> 1 (refused) or on."""
    out, pending, mask = [], None, None
    for r in trace:
        kind = kind_of(r)
        if kind == 'practice_leave':
            mask = num(cap(r, 1))
            continue
        if kind != 'practice_state':
            continue
        regs = r.get('registers') or {}
        new, old = num(regs.get('x8')), num(regs.get('x9'))
        frame = num(r.get('frame')) or 0
        if new == 4 and old != 4:
            pending, mask = frame, None
        elif old == 4 and pending is not None:
            out.append((pending, 'refused' if new == 1 else 'accepted', mask))
            pending = None
    if pending is not None:
        out.append((pending, 'open', mask))
    return out


SYNC_END = re.compile(r'sync_end\b.*\bdesync_seen=(\d+)')
SESSION = re.compile(r'\bsession slot=\d+ kind=online')


def desync_clean(text):
    """(clean, sessions, sync_ends): clean when there was a session and every sync_end has desync_seen=0."""
    ends = [int(m.group(1)) for m in SYNC_END.finditer(text)]
    sessions = len(SESSION.findall(text))
    return bool(ends) and all(e == 0 for e in ends), sessions, len(ends)


def pairs(items):
    out = {}
    for item in items:
        key, _, value = item.partition('=')
        out[key] = value
    return out


def main(argv=None):
    p = argparse.ArgumentParser()
    p.add_argument('--trace', action='append', default=[], help='<instance>=<trace.jsonl>')
    p.add_argument('--detector', action='append', default=[], help='<instance>=<events.log>')
    p.add_argument('--json', action='store_true')
    args = p.parse_args(argv)
    result = {}
    for key, path in pairs(args.trace).items():
        trace = list(records(open(path, encoding='utf-8', errors='replace')))
        result.setdefault(key, {})['round_starts'] = round_start_paths(trace)
        result[key]['leaves'] = leave_outcomes(trace)
    for key, path in pairs(args.detector).items():
        clean, sessions, ends = desync_clean(Path(path).read_text(encoding='utf-8', errors='replace'))
        result.setdefault(key, {})['desync'] = {'clean': clean, 'sessions': sessions, 'sync_ends': ends}
    if args.json:
        print(json.dumps(result, indent=2))
        return 0
    for key, facts in sorted(result.items()):
        for frame, path, counter in facts.get('round_starts', []):
            print(f'{key} round_start frame={frame} path={path} counter={counter}')
        for frame, outcome, mask in facts.get('leaves', []):
            print(f'{key} leave frame={frame} outcome={outcome} mask={mask}')
        if 'desync' in facts:
            d = facts['desync']
            print(f'{key} desync clean={d["clean"]} sessions={d["sessions"]} sync_ends={d["sync_ends"]}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
