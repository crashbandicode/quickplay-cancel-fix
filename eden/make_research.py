"""Rows for the 14:55Z research boots (V: both consoles without QCF; P: a with QCF v0.2.0), the same rows in both.
Both consoles run the CPU pilot with auto_rematch=on, so every "Rematch?" is answered Yes natively by both.
  m1    a's first pick of the boot joins b's room from the character select; the match is played (I1: the first
        match's RMC calls, vanilla in V and v0.2.0 in P).
  ry1   both answer Yes; b presses B at each phase of the re-entry (I2); GIF of a's screen; the rematch match is
        played if it starts (I1: the native rematch re-entry, count >= 2, in both boots).
  ry2   both answer Yes again; b presses Plus at each phase and takes Leave Battle if shown (I2); GIF of a's screen.
  ry3   only if ry2 ended in a match: the same Plus tries again, no GIF.
  m2    b hosts from the character select again, a joins; the match is played; ry4/ry5 as ry2/ry3.
Usage: make_research.py <rows-rm02 folder> <dst folder>
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


def gif(name, on, length, reason, inner):
    return {'on': on, 'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': inner}}


def tries(tag, button, take_gif=True, skip_if_left=None, leave=True):
    inner = [step('rematch_tries', 'b', tag=tag, button=button, cap_s=60, leave=leave,
                  **({'skip_if_left': skip_if_left} if skip_if_left else {}))]
    steps = ([gif(f'{tag}-a-rematch', 'a', 20, f"I2 evidence: a's screen while b presses {button} at each phase of "
                  "the rematch re-entry; capped at 20 s so the rematch match that may follow isn't left idle during "
                  "the encode; raw take", inner)] if take_gif else inner)
    return steps + [step('battle_if_matched', 'a', tag=tag),
                    step('where_are', 'a', tag=f'{tag}-after', wait_s=4.0)]


first = [s for s in by['t1-throwaway']['steps'] if s.get('run') not in ('steps.py:ready_and_wait_session',
                                                                        'steps.py:play_out')]
new = [by[i] for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay', 'r01-b-waits')]
new += [
    {'id': 'm1', 'start': 'b-searching', 'end': 'results', 'budget_min': 8, 'steps': first + [
        step('ready_and_wait_session', 'a', tag='m1'), step('battle', 'a', tag='m1')]},
    {'id': 'ry1', 'start': 'results', 'end': 'results-or-left', 'budget_min': 9, 'steps': tries('ry1', 'b', leave=False)},
    {'id': 'ry2', 'start': 'results-or-left', 'end': 'results-or-left', 'budget_min': 9, 'steps': tries('ry2', 'plus')},
    {'id': 'ry3', 'start': 'results-or-left', 'end': 'left', 'budget_min': 9,
     'steps': tries('ry3', 'plus', take_gif=False, skip_if_left='ry2')},
    {'id': 'm2', 'start': 'left', 'end': 'results', 'budget_min': 10, 'steps': [
        step('leave_if_searching', 'a', tag='m2a'),
        step('leave_if_searching', 'b', tag='m2b'),
        step('ensure_searching', 'b', tag='m2w'),
        step('css_to_banner', 'a'),
        step('ready_and_wait_session', 'a', tag='m2'),
        step('battle', 'a', tag='m2')]},
    {'id': 'ry4', 'start': 'results', 'end': 'results-or-left', 'budget_min': 9, 'steps': tries('ry4', 'plus')},
    {'id': 'ry5', 'start': 'results-or-left', 'end': 'left', 'budget_min': 9,
     'steps': tries('ry5', 'plus', take_gif=False, skip_if_left='ry4')},
    dict(by['z-a'], start='left'), by['z-b'],
]
rows['rows'] = new
rows['about'] = ("QCF research (primary 14:55Z, I1/I2) on the research LAN runtime (each RMC request body logged): the "
                 "CPU pilot on both consoles answers every Rematch? Yes; m1/m2 first matches from the character select, "
                 "ry* the rematch re-entries with b pressing B or Plus at each phase. Boot V without QCF, boot P with "
                 "QCF v0.2.0 on a.")
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps([r['id'] for r in new]))
