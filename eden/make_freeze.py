"""Rows for the freeze A/B (primary 17:35Z/17:59Z): each arm is one boot doing exactly what qcf-srm12 did up to its
freeze, m1 (a's first pick joins b's room from the character select; the match is played) and ry1 (both answer
Rematch Yes through the CPU pilot; b presses B at each re-entry phase, a's screen is recorded; the rematch match is
played to its end), then the instances are closed. The rows are the same in every arm; the NROs differ.
With --gdb, b boots with Eden's GDB stub (steps.py:gdb_on before its game loads, gdb_attached after; b untraced) and
`gdbdump.py` (the rc6 hang lane's helper) is copied beside the rows.
Usage: make_freeze.py <rows-rs01 folder> <dst folder> [--gdb]
"""
import json
import shutil
import sys
from pathlib import Path

GDBDUMP = Path('/home/intpa/smash-training-lobby/artifacts/phase23/rc6-eden/rows-hang-qp/gdbdump.py')
src, dst = Path(sys.argv[1]), Path(sys.argv[2])
gdb = '--gdb' in sys.argv[3:]
if dst.exists():
    shutil.rmtree(dst)
shutil.copytree(src, dst)
rows = json.loads((dst / 'rows.json').read_text())
by = {r['id']: r for r in rows['rows']}
new = [by[i] for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay', 'r01-b-waits', 'm1', 'ry1')]
new += [dict(by['z-a'], start='results-or-left'), by['z-b']]
if gdb:
    load = dict(new[0])
    steps = list(load['steps'])
    at = next(i for i, s in enumerate(steps) if s.get('call') == 'game.load' and s.get('on') == 'b')
    steps.insert(at, {'run': 'steps.py:gdb_on', 'on': 'b', 'check': True, 'args': {}})
    after = next(i for i, s in enumerate(steps) if s.get('gate') == 'loaded' and s.get('on') == 'b')
    steps.insert(after + 1, {'run': 'steps.py:gdb_attached', 'on': 'b', 'check': True, 'args': {}})
    load['steps'] = steps
    new[0] = load
    shutil.copy2(GDBDUMP, dst / 'gdbdump.py')
    # The candidate arms show the tab: m1's Ready also peeks into the waiting menu once a reported 7.
    m1 = dict(new[5])
    m1['steps'] = [dict(s, run='steps.py:ready_peek_and_wait_session') if s.get('run') == 'steps.py:ready_and_wait_session'
                   else s for s in m1['steps']]
    assert any(s.get('run') == 'steps.py:ready_peek_and_wait_session' for s in m1['steps'])
    new[5] = m1
    # b is untraced, so the rematch is followed from a (a's answer in its trace, the match in its detector); b no
    # longer presses B through the re-entry.
    # The pass criteria (primary 19:17Z): Leave Battle during the opponent's load on a first match from the CSS, and
    # a clean leave back to the CSS. l1 replaces r01: a hosts alone, b joins a's room from the CSS, a pauses during
    # b's load and takes Leave Battle (pause_during_join on a's own screen and trace); b stays, searching alone as
    # host, so m1 is again a joining b's room, from the CSS.
    def st(run, on, **args):
        return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}
    new[4] = {'id': 'l1-host-leave', 'start': 'quickplay-menu', 'end': 'b-searching', 'budget_min': 8, 'steps': [
        st('reach_css_tile', 'a'), {'gate': 'css-tile', 'on': 'a'}, {'play': 'pick', 'on': 'a'},
        {'gate': 'ready-banner', 'on': 'a'}, st('ready_and_wait_searching', 'a', tag='l1w'),
        st('reach_css_tile', 'b'), {'gate': 'css-tile', 'on': 'b'}, {'play': 'pick', 'on': 'b'},
        {'gate': 'ready-banner', 'on': 'b'}, st('ready_only', 'b', tag='l1b'),
        st('pause_during_join', 'a', tag='l1', join_cap_s=150, cap_s=60, leave=True, wait_p2=True),
        st('expect_left', 'a', tag='l1')]}
    m1 = dict(new[5])
    m1['steps'] = [st('css_to_banner', 'a'), st('ready_peek_and_wait_session', 'a', tag='m1'), st('battle', 'a', tag='m1')]
    new[5] = m1
    ry1 = dict(new[6])
    ry1['steps'] = [{'run': 'steps.py:wait_answered', 'on': 'a', 'check': True, 'args': {'tag': 'ry1', 'cap_s': 120}},
                    {'run': 'steps.py:rematch_match', 'on': 'a', 'check': True, 'args': {'tag': 'ry1', 'wait_s': 90}}]
    new[6] = ry1
rows['rows'] = new
rows['about'] = ("QCF freeze A/B (primary 17:35Z/17:59Z): qcf-srm12's m1 and ry1 (the first native rematch match "
                 "after a first match from the character select), played to the end; the arms differ only in a's "
                 "NROs (none, QCF v0.2.0, candidate R, candidate N); the research LAN runtime"
                 + ("; b with Eden's GDB stub (untraced)." if gdb else "."))
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps([r['id'] for r in new]), 'gdb' if gdb else '')
