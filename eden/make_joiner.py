"""Rows for the joiner test (primary 13:46Z, the operator's question: can the console that JOINS a room cancel before
the match fully loads?). a runs QCF v0.2.0 (release), b is vanilla. No code change: v0.2.0 already raises a's own
slot count to 2 in every lobby it enters from the character select, also when it joins.
  oj-1      a's first pick of the boot: a joins the room b has held since r01 (a late join).
  oj-2..N   b hosts fresh from the character select, a readies and joins: from a's own entry into the room, a presses
            Plus and looks every 0.35 s (its own state, the member state it reported, the menu gate's verdict); with
            Leave Battle there, a takes it (Up, A, Yes). A try that becomes a match is played out.
  vj-1..M   the control: a hosts from the character select, b (vanilla) joins and does the same.
Each try ends with where both are (a at the character select and b searching alone, after a leave).
Usage: make_joiner.py <rows-rm02 folder> <dst folder> [joiner tries=4] [control tries=2]
"""
import json
import shutil
import sys
from pathlib import Path

src, dst = Path(sys.argv[1]), Path(sys.argv[2])
tries = int(sys.argv[3]) if len(sys.argv) > 3 else 4
controls = int(sys.argv[4]) if len(sys.argv) > 4 else 2
if dst.exists():
    shutil.rmtree(dst)
shutil.copytree(src, dst)
rows = json.loads((dst / 'rows.json').read_text())
by = {r['id']: r for r in rows['rows']}


def step(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def tail(tag, joiner):
    return [step('pause_own_load', joiner, tag=tag, room_cap_s=150, cap_s=90, leave=True),
            step('after_soft', 'a', tag=tag),
            step('where_are', 'a', tag=f'{tag}-after', wait_s=6.0)]


new = [by[i] for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay', 'r01-b-waits')]
first = [s for s in by['t1-throwaway']['steps'] if s.get('run') not in ('steps.py:ready_and_wait_session',
                                                                        'steps.py:play_out')]
new.append({'id': 'oj-1', 'start': 'b-searching', 'end': 'css-or-searching', 'budget_min': 10,
            'steps': first + [step('ready_until_searching', 'a', tag='oj1r')] + tail('oj1', 'a')})
for n in range(2, tries + 1):
    new.append({'id': f'oj-{n}', 'start': 'css-or-searching', 'end': 'css-or-searching', 'budget_min': 10, 'steps': [
        step('leave_if_searching', 'a', tag=f'oj{n}a'),
        step('leave_if_searching', 'b', tag=f'oj{n}b'),
        step('ensure_searching', 'b', tag=f'oj{n}w'),
        step('ready_until_searching', 'a', tag=f'oj{n}r')] + tail(f'oj{n}', 'a')})
for n in range(1, controls + 1):
    new.append({'id': f'vj-{n}', 'start': 'css-or-searching', 'end': 'css-or-searching', 'budget_min': 10, 'steps': [
        step('leave_if_searching', 'a', tag=f'vj{n}a'),
        step('leave_if_searching', 'b', tag=f'vj{n}b'),
        step('ensure_searching', 'a', tag=f'vj{n}w'),
        step('ready_until_searching', 'b', tag=f'vj{n}r')] + tail(f'vj{n}', 'b')})
new += [dict(by['z-a'], start='css-or-searching'), by['z-b']]
rows['rows'] = new
rows['about'] = ("quickplay-cancel-fix joiner test (primary 13:46Z) on the 0x2C-fixed LAN runtime: a runs QCF v0.2.0 "
                 "(release), b vanilla. oj: a joins b's room from the character select and presses Plus through its "
                 "own load (Leave Battle taken when shown); vj: the control, b (vanilla) joins a's room the same way.")
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps([r['id'] for r in new]))
