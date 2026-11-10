"""Verdicts for a QCF Eden run from each instance's trace stream, desync-detector log, the plugin's events.log and
the harness's pause tries.

Usage: verdict.py [--trace a=<trace.jsonl> ...] [--detector a=<events.log> ...] [--plugin a=<events.log> ...]
                  [--tries <run out/ folder>] [--json]

- round starts: every host round start (`.1566af4`) with the path that started it, `countdown` (the room
  countdown had run out) or `full_room` (the room was full for 180 frames), from the waiting battle's
  full-room counter captured there;
- leave attempts: every PracticeScene leave (state 1 -> 4) and its outcome, `accepted` (4 -> anything but 1)
  or `refused` (4 -> 1, a battle round had started), with the round ack mask captured at the leave state;
- desync: the detector's `sync_end` lines, clean when each says `desync_seen=0`;
- sessions: each online session since the detector's last install line, `clean` (a sync_end with desync_seen=0),
  `desync` (a sync_end with desync_seen > 0), `cut` (a frame gap and no sync_end: the session stopped on both
  consoles without ending) or `open`;
- plugin: the install result and every `adopted slot=S count=B->A` line (diag build);
- tries: each `*-try-leave.json` (a pause during the opponent's load): whether Leave Battle was shown, when it
  was first seen, and the outcome (`left`, `matched`, `none`).
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


SESSION_LINE = re.compile(r'\bsession slot=(\d+) kind=(\w+)')
FRAME_GAP = re.compile(r'\bframe_gap kind=\w+ slot=\d+ frame=(\d+)')
SESSION_END = re.compile(r'\bsync_end trigger=(\w+) kind=(\d+) frame=(\d+)\b.*\bdesync_seen=(\d+)')
INSTALL = re.compile(r'\bplugin=[\w.-]+/\S+')


def online_sessions(text):
    """[(slot, status, end, gap)] for each online session after the log's last install line. `end` is
    `trigger/kind@frame` or None, `gap` the first frame gap's frame or None."""
    lines = text.splitlines()
    start = max((i for i, line in enumerate(lines) if INSTALL.search(line)), default=-1) + 1
    out, cur = [], None
    for line in lines[start:]:
        m = SESSION_LINE.search(line)
        if m:
            cur = {'slot': int(m.group(1)), 'online': m.group(2) == 'online', 'gap': None, 'end': None,
                   'desync': None}
            out.append(cur)
            continue
        if cur is None:
            continue
        m = FRAME_GAP.search(line)
        if m and cur['gap'] is None:
            cur['gap'] = int(m.group(1))
        m = SESSION_END.search(line)
        if m and cur['end'] is None:
            cur['end'] = f'{m.group(1)}/{m.group(2)}@{m.group(3)}'
            cur['desync'] = int(m.group(4))
    result = []
    for s in out:
        if not s['online']:
            continue
        if s['end'] is not None:
            status = 'clean' if s['desync'] == 0 else 'desync'
        else:
            status = 'cut' if s['gap'] is not None else 'open'
        result.append((s['slot'], status, s['end'], s['gap']))
    return result


INSTALLED = re.compile(r'\bplugin=quickplay-cancel-fix/(\S+)(.*?)\bresult=(\w+)')
ADOPTED = re.compile(r'\badopted slot=(\d+) count=(\d+)->(\d+)')


def plugin_log(text):
    """(installs, adopted): installs [(version, build, result)], adopted [(slot, before, after)]."""
    installs = [(m.group(1), 'diag' if 'build=diag' in m.group(2) else 'release', m.group(3))
                for m in INSTALLED.finditer(text)]
    adopted = [tuple(int(g) for g in m.groups()) for m in ADOPTED.finditer(text)]
    return installs, adopted


def leave_try(data):
    """(tag, tab_shown, tab_first_s, outcome) from one `*-try-leave.json` (the harness's pause during the
    opponent's load): the tab counts as shown when a look with the menu open saw it."""
    looks = [k for k in data.get('looks') or [] if isinstance(k, dict)]
    seen = [k.get('t') for k in looks if k.get('tab') and k.get('what') != 'leave' and k.get('menu')]
    return data.get('tag'), bool(seen), (min(seen) if seen else None), data.get('outcome')


def leave_tries(folder):
    """leave_try for every `*-try-leave.json` under `folder`, in path order."""
    return [leave_try(json.loads(p.read_text())) for p in sorted(Path(folder).rglob('*-try-leave.json'))]


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
    p.add_argument('--plugin', action='append', default=[], help='<instance>=<quickplay-cancel-fix events.log>')
    p.add_argument('--tries', help="a run's out/ folder with the *-try-leave.json files")
    p.add_argument('--json', action='store_true')
    args = p.parse_args(argv)
    result = {}
    for key, path in pairs(args.plugin).items():
        installs, adopted = plugin_log(Path(path).read_text(encoding='utf-8', errors='replace'))
        result.setdefault(key, {})['plugin'] = {'installs': installs, 'adopted': adopted}
    if args.tries:
        result['tries'] = leave_tries(args.tries)
    for key, path in pairs(args.trace).items():
        trace = list(records(open(path, encoding='utf-8', errors='replace')))
        result.setdefault(key, {})['round_starts'] = round_start_paths(trace)
        result[key]['leaves'] = leave_outcomes(trace)
    for key, path in pairs(args.detector).items():
        text = Path(path).read_text(encoding='utf-8', errors='replace')
        clean, sessions, ends = desync_clean(text)
        result.setdefault(key, {})['desync'] = {'clean': clean, 'sessions': sessions, 'sync_ends': ends}
        result[key]['sessions'] = online_sessions(text)
    if args.json:
        print(json.dumps(result, indent=2))
        return 0
    for tag, shown, first, outcome in result.pop('tries', []):
        print(f'try {tag} tab={"shown" if shown else "hidden"} first_s={first} outcome={outcome}')
    for key, facts in sorted(result.items()):
        for version, build, res in facts.get('plugin', {}).get('installs', []):
            print(f'{key} plugin version={version} build={build} result={res}')
        for slot, before, after in facts.get('plugin', {}).get('adopted', []):
            print(f'{key} adopted slot={slot} count={before}->{after}')
        statuses = [s[1] for s in facts.get('sessions', [])]
        if 'sessions' in facts:
            counts = ' '.join(f'{k}={statuses.count(k)}' for k in ('clean', 'desync', 'cut', 'open'))
            print(f'{key} sessions {len(statuses)}: {counts}')
            for slot, status, end, gap in facts['sessions']:
                print(f'{key}   slot={slot} {status} end={end} gap={gap}')
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
