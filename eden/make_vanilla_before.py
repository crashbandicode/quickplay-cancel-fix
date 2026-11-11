"""Rows for the 0.3.0 README "before" take with no plugin on either console (the operator's 00:06Z correction: the
before must be vanilla): b searches alone; a, without QCF, readies from the character select and joins b's room; from
a's waiting area a take of a's screen while a presses Pause as it comes (`hold_join_watch`, leave=True: it would take
Leave Battle if the game offered it); then the match is played as far as it goes and the end rows check both consoles.
Usage: make_vanilla_before.py <rows-hd04 folder> <dst folder>
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


def st(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, on, length, reason, inner):
    return {'on': on, 'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': inner}}


rows['rows'] = [by[i] for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay', 'r01-b-waits')] + [
    {'id': 'vb', 'start': 'b-searching', 'end': 'results', 'budget_min': 8, 'steps': [
        st('reach_css_tile', 'a'), {'gate': 'css-tile', 'on': 'a'}, {'play': 'pick', 'on': 'a'},
        {'gate': 'ready-banner', 'on': 'a'},
        st('hold_join_ready', 'a', tag='vb', expect_held=False),
        gif('before-joiner-vanilla', 'a', 20, "README 'before' (0.3.0): no plugin on either console; a has joined b's "
            "room from the character select; from its waiting area, Pause as it comes until the match starts; raw take",
            [st('hold_join_watch', 'a', tag='vb', leave=True, cap_s=40)]),
        st('expect_hold', 'a', tag='vb', held=False, menu=None, tab=False),
        st('battle_soft', 'a', tag='vb')]},
    dict(by['z-a'], start='results'), by['z-b'],
]
rows['about'] = ("QCF 0.3.0 README 'before' (operator 00:06Z): both consoles without QCF; a joins b's room from the "
                 "character select and presses Pause as it comes; the research LAN runtime, both traced.")
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps([r['id'] for r in rows['rows']]))
