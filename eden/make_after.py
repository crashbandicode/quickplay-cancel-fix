"""Pair rows for the after-fix boot: a runs the QCF plugin (0.2.0, the rematch count for a character-select lobby),
b is vanilla. q00-q03, r01 (b waits), t1 (throwaway first match), then:
  before  b hosts from the CSS (vanilla), a joins; b pauses during a's load: no Leave Battle (README 'before' GIF),
          the match is played out.
  after   a hosts from the CSS (with the plugin), b joins; a pauses during b's load: Leave Battle, a leaves the game's
          own way (README 'after' GIF); then where both are (b should be alone in its waiting area, searching again).
  after-b b leaves its waiting area (Leave Battle, alone) to the CSS.
  play    a hosts from the CSS again, b joins; a opens the menu during b's load (tab expected) and closes it; the match
          is played out normally (no desync with the plugin when nobody leaves).
then z.
Usage: make_after.py <rows-rm02 folder> <dst folder> [repeat=1]
  then repeat x (join-N: the fix as the joiner, host-N: as the host), each match played out.
"""
import json
import shutil
import sys
from pathlib import Path

src, dst = Path(sys.argv[1]), Path(sys.argv[2])
if dst.exists():
    shutil.rmtree(dst)
shutil.copytree(src, dst)
rows = json.loads((dst / 'rows.json').read_text())
by = {r['id']: r for r in rows['rows']}


def step(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, reason, inner, length=110):
    return {'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': inner}}


new = [by[i] for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay', 'r01-b-waits', 't1-throwaway')]
new += [
    {'id': 'before', 'start': 'css-or-searching', 'end': 'css', 'budget_min': 14, 'steps': [
        step('leave_if_searching', 'a', tag='bfa'),
        step('leave_if_searching', 'b', tag='bfb'),
        gif('before-b-css-lobby', "README 'before': vanilla console b readies from the character select and hosts; a "
            "joins; b pauses during a's loading: no Leave Battle; raw take",
            [step('ensure_searching', 'b', tag='bfw'),
             step('ready_until_searching', 'a', tag='bfr'),
             step('pause_during_join', 'b', tag='bf', join_cap_s=150, cap_s=60, wait_p2=True, leave=True)]),
        step('after_soft', 'a', tag='bf'),
        step('where_are', 'a', tag='bf-after', wait_s=3.0)]},
    {'id': 'after', 'start': 'css', 'end': 'a-left', 'budget_min': 12, 'steps': [
        step('leave_if_searching', 'a', tag='afa'),
        step('leave_if_searching', 'b', tag='afb'),
        gif('after-a-css-lobby', "README 'after': console a (QCF plugin) readies from the character select and hosts; "
            "b joins; a pauses during b's loading: Leave Battle, Yes, back to the character select; raw take",
            [step('ensure_searching', 'a', tag='afw'),
             step('ready_until_searching', 'b', tag='afr'),
             step('pause_during_join', 'a', tag='af', join_cap_s=150, cap_s=60, wait_p2=True, leave=True),
             step('where_are', 'a', tag='af-css', wait_s=6.0)]),
        step('where_are', 'a', tag='af-after', wait_s=8.0)]},
    {'id': 'after-b', 'start': 'a-left', 'end': 'css', 'budget_min': 6, 'steps': [
        step('settle', 'b', wait_s=10.0),
        step('where_are', 'a', tag='af-b-waits', wait_s=1.0),
        step('leave_if_searching', 'b', tag='afbl'),
        step('where_are', 'a', tag='af-b-left', wait_s=3.0)]},
    {'id': 'play', 'start': 'css', 'end': 'css', 'budget_min': 14, 'steps': [
        step('ensure_searching', 'a', tag='plw'),
        step('ready_until_searching', 'b', tag='plr'),
        step('pause_during_join', 'a', tag='pl', join_cap_s=150, cap_s=60, wait_p2=True, leave=False),
        step('after_soft', 'a', tag='pl'),
        step('where_are', 'a', tag='pl-after', wait_s=3.0)]},
]
# Repeated matches with the fix: as the joiner (b vanilla hosts) and as the host, each played out (the A/B for the
# environment's frame-gap error, xrm07 before).
for n in range(1, 1 + int(sys.argv[3]) if len(sys.argv) > 3 else 1):
    new += [
        {'id': f'join-{n}', 'start': 'css', 'end': 'css', 'budget_min': 12, 'steps': [
            step('leave_if_searching', 'a', tag=f'j{n}a'),
            step('leave_if_searching', 'b', tag=f'j{n}b'),
            step('ensure_searching', 'b', tag=f'j{n}w'),
            step('ready_until_searching', 'a', tag=f'j{n}r'),
            step('pause_during_join', 'b', tag=f'j{n}', join_cap_s=150, cap_s=60, wait_p2=True, leave=False),
            step('after_soft', 'a', tag=f'j{n}'),
            step('where_are', 'a', tag=f'j{n}-after', wait_s=3.0)]},
        {'id': f'host-{n}', 'start': 'css', 'end': 'css', 'budget_min': 12, 'steps': [
            step('leave_if_searching', 'a', tag=f'h{n}a'),
            step('leave_if_searching', 'b', tag=f'h{n}b'),
            step('ensure_searching', 'a', tag=f'h{n}w'),
            step('ready_until_searching', 'b', tag=f'h{n}r'),
            step('pause_during_join', 'a', tag=f'h{n}', join_cap_s=150, cap_s=60, wait_p2=True, leave=False),
            step('after_soft', 'a', tag=f'h{n}'),
            step('where_are', 'a', tag=f'h{n}-after', wait_s=3.0)]},
    ]
new += [dict(by['z-a'], start='css'), by['z-b']]
rows['rows'] = new
rows['about'] = ("quickplay-cancel-fix 0.2.0 after-fix pair on the 0x2C-fixed LAN runtime: a runs the plugin (diag build), "
                 "b is vanilla; QCF trace points. before (b's CSS lobby, no tab), after (a's CSS lobby, tab, native "
                 "leave, b searches again), play (a's CSS lobby, the match played out).")
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps([r['id'] for r in new]))
