"""Readable timeline of a QCF Eden run's trace stream (Eden guest trace JSONL, schema 1).

Usage: timeline.py <trace.jsonl> [--from-frame N] [--to-frame N]
Prints one line per record of the QCF trace points (eden/make_config.py POINTS): the Quickplay parent and
IngameCallSequenceScene state changes with state names, PracticeScene states and results, scene requests, and
the online service's battle-start round (start, mask writes, messages >= 0xbe).
"""
import argparse
import json
import sys

PARENT = {0: 'exit', 1: 'leave', 2: 'exit2', 3: 'OnlineMeleeAnyScene', 4: 'CharaSelectScene', 5: 'search-start',
          6: 'bookkeeping6', 7: 'matchmaking', 8: 'IngameCallSequenceScene', 9: 'unlock-check', 10: 'state10',
          11: 'error11', 12: 'restore12'}
CHILD = {0: 'prep', 1: 'leave', 2: 'SyncParameterScene', 3: 'PracticeScene', 4: 'MeleeScene', 8: 'session-leave',
         12: 'stage-prep12'}
PRACTICE = {0: 'init', 1: 'running', 2: 'exit', 3: 'net-event', 4: 'leave'}
KINDS = {
    'main+0x236b4ac': 'parent', 'main+0x23191bc': 'child', 'main+0x23105bc': 'practice_result',
    'main+0x37266d0': 'scene', 'main+0x18514bc': 'practice_state', 'main+0x185129c': 'match_received',
    'main+0x185113c': 'practice_msg67', 'main+0x18518b0': 'practice_leave', 'main+0x1851310': 'quit_request',
    'main+0x16a1df0': 'round_start', 'main+0x16b9830': 'round_ensure', 'main+0x16a2120': 'mask_abort',
    'main+0x16a21e8': 'mask_notready', 'main+0x168b500': 'mask_reset', 'main+0x16b2d6c': 'mask_store_b2d6c',
    'main+0x169eadc': 'mask_store_9eadc', 'main+0x1892048': 'mask_store_92048', 'main+0x1685cb0': 'send',
    'main+0x1685ac0': 'send_to', 'main+0x1566af4': 'host_round_start', 'main+0x15669c4': 'host_ready_check',
    'main+0x1564088': 'countdown_read', 'main+0x1563454': 'wait_pre_reset', 'main+0x1563650': 'countdown_reset',
    'main+0x15648d0': 'wait_enter', 'main+0x1566758': 'host_branch',
}


def kind_of(record):
    for loc in (record.get('locations') or {}).values():
        if loc in KINDS:
            return KINDS[loc]
    return None


def num(value):
    try:
        return int(value, 16) if isinstance(value, str) and value.startswith('0x') else int(value)
    except (TypeError, ValueError):
        return None


def cap(record, i):
    caps = record.get('captures') or []
    if i >= len(caps):
        return None
    c = caps[i]
    if c.get('error'):
        return 'err'
    if c.get('type') == 'c-string':
        return c.get('value')
    return c.get('value') if c.get('type') != 'bytes' else c.get('hex')


def describe(kind, r):
    regs = r.get('registers') or {}
    x8, x9 = num(regs.get('x8')), num(regs.get('x9'))
    if kind == 'parent':
        return f'parent {PARENT.get(x9, x9)} -> {PARENT.get(x8, x8)} level={cap(r, 0)}'
    if kind == 'child':
        return (f'child {CHILD.get(x9, x9)} -> {CHILD.get(x8, x8)} level={cap(r, 0)} flags198={cap(r, 1)} '
                f'mode={cap(r, 2)}')
    if kind == 'practice_result':
        return f'PracticeScene result {x8}'
    if kind == 'scene':
        return f'scene request {cap(r, 0)!r} (from {cap(r, 1)!r})'
    if kind == 'practice_state':
        return f'practice {PRACTICE.get(x9, x9)} -> {PRACTICE.get(x8, x8)} f4={cap(r, 0)} mask={cap(r, 1)}'
    if kind == 'match_received':
        return f'practice match received mask={cap(r, 0)}'
    if kind == 'practice_msg67':
        return f'practice net event type={cap(r, 0)} -> state 3'
    if kind == 'practice_leave':
        return f'practice leave state age={cap(r, 0)} mask={cap(r, 1)}'
    if kind == 'quit_request':
        return f'Leave Battle requested (lr={regs.get("x30")})'
    if kind in ('round_start', 'round_ensure'):
        rest = [cap(r, i) for i in range(len(r.get('captures') or []))]
        return f'{kind} x1={regs.get("x1")} lr={regs.get("x30")} caps={rest}'
    if kind.startswith('mask'):
        return f'{kind} {regs}'
    if kind in ('host_round_start', 'host_ready_check', 'countdown_read', 'wait_pre_reset', 'countdown_reset',
                'wait_enter', 'host_branch'):
        rest = [cap(r, i) for i in range(len(r.get('captures') or []))]
        return f'{kind} regs={regs} caps={rest}'
    if kind in ('send', 'send_to'):
        msg = num(regs.get('x1')) if kind == 'send' else num(regs.get('x2'))
        return f'{kind} msg={msg:#x} lr={regs.get("x30")}' if msg is not None else f'{kind} {regs}'
    return kind


def main():
    p = argparse.ArgumentParser()
    p.add_argument('trace')
    p.add_argument('--from-frame', type=int, default=0)
    p.add_argument('--to-frame', type=int, default=1 << 62)
    args = p.parse_args()
    for line in open(args.trace, encoding='utf-8', errors='replace'):
        try:
            r = json.loads(line)
        except ValueError:
            continue
        if r.get('event') == 'loss':
            print(f'-- loss {r.get("records")} records')
            continue
        kind = kind_of(r)
        frame = num(r.get('frame')) or 0
        if kind is None or not (args.from_frame <= frame <= args.to_frame):
            continue
        print(f'{frame:>8} {describe(kind, r)}')


if __name__ == '__main__':
    sys.exit(main())
