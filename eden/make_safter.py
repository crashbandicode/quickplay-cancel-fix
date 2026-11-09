"""Rows for the "after" boot on the Surface pair (a with the QCF NRO, b vanilla), from the obs01 rows' gates, segments
and hooks (primary 2026-11-09 18:40Z):

  f1  a's first match of a Quickplay session, joining b whose countdown has run out: a leaves in the hold
      (expect 'left'); b keeps searching (the native member leave)          -> the README "after" GIF take
  f2  a re-enters Quickplay (a new session), first match again against b waiting long, no leave: the match
      starts and is fought to its end on both (detector sync_end, desync_seen=0), then both decline the rematch
  s1  the same session's next match, b's countdown fresh (b readies first, a at once): native, no hold;
      a tries the leave in the native full-room wait (the leave-menu timing check)
  z   Unmapped 0, console clean, QCF events

Usage: make_safter.py <eden/obs01> <dst folder> [--cap 80]
"""
import argparse
import json
import shutil
from pathlib import Path

INSTALLED = r'plugin=quickplay-cancel-fix/\S+ build=diag [^\n]*result=installed heap_allocations=0'


def leave_try(tag, host_wait_s=None, try_leave=True, expect=None, cap=80):
    args = {'tag': tag, 'cap_s': cap, 'try_leave': try_leave}
    if host_wait_s is not None:
        args['host_wait_s'] = host_wait_s
    if expect:
        args['expect'] = expect
    return {'run': 'steps.py:ready_and_try_leave', 'on': 'a', 'check': True, 'args': args}


def build(src, cap):
    rows = json.loads((src / 'rows.json').read_text())
    keep = {r['id']: r for r in rows['rows']}
    rows['about'] = ("quickplay-cancel-fix after boot (Surface pair; a with the QCF diag NRO, b vanilla; detector and "
                     "tile cursor on both): a's first match of a Quickplay session as the joiner of b's room after b's "
                     "countdown ran out, left in the hold (f1) and played (f2); the session's next match with b's "
                     "countdown fresh, native (s1). Trace points on both.")
    rows['scope'] = 'QCF after rows on the Surface Quickplay pair (route from rc6-bg-qp6).'
    rows['setup'][0]['ref'] = ("pair config: instances a and b (rc6-bg-qp6's route on the Surface): detector + tile "
                               "cursor, the QCF diag NRO on a")
    rows['gates']['qcf-installed'] = {'kind': 'log', 'source': 'qcf_log', 'whole_file': True, 'cap': 120,
                                      'pattern': INSTALLED}
    load = keep['q00-load']
    at = max(i for i, s in enumerate(load['steps']) if s.get('gate') == 'installed') + 1
    load['steps'][at:at] = [{'gate': 'qcf-installed', 'on': 'a'}]
    new = [
        keep['q00-load'], keep['q01-main'], keep['q02-online'], keep['q03-quickplay'], keep['r01-b-waits'],
        {'id': 'f1-first-leave', 'start': 'b-searching', 'end': 'tried', 'budget_min': 6, 'steps': [
            {'run': 'steps.py:reach_css_tile', 'on': 'a', 'check': True}, {'gate': 'css-tile', 'on': 'a'},
            {'play': 'pick', 'on': 'a'}, {'gate': 'ready-banner', 'on': 'a'},
            {'gif': {'name': 'f1-a-first-after', 'preset': 'docs', 'length': 100,
                     'reason': "raw take for the README 'after' GIF; trimmed under 10 s after review",
                     'steps': [leave_try('f1', host_wait_s=30, expect='left', cap=cap)]}}]},
        {'id': 'f1-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 3, 'steps': [
            {'run': 'steps.py:after_try', 'on': 'a', 'check': True, 'args': {'tag': 'f1'}}]},
        {'id': 'f2-reenter-play', 'start': 'css-or-searching', 'end': 'tried', 'budget_min': 7, 'steps': [
            {'run': 'steps.py:leave_css_to_quickplay', 'on': 'a', 'check': True},
            {'run': 'steps.py:ensure_searching', 'on': 'b', 'check': True, 'args': {'tag': 'w2'}},
            {'run': 'steps.py:reach_css_tile', 'on': 'a', 'check': True}, {'gate': 'css-tile', 'on': 'a'},
            {'play': 'pick', 'on': 'a'}, {'gate': 'ready-banner', 'on': 'a'},
            leave_try('f2', host_wait_s=30, try_leave=False, expect='matched', cap=cap)]},
        {'id': 'f2-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 8, 'steps': [
            {'run': 'steps.py:after_try', 'on': 'a', 'check': True, 'args': {'tag': 'f2'}}]},
        {'id': 's1-second-fresh', 'start': 'css-or-searching', 'end': 'tried', 'budget_min': 7, 'steps': [
            {'run': 'steps.py:css_to_banner', 'on': 'a', 'check': True},
            {'run': 'steps.py:ensure_searching', 'on': 'b', 'check': True, 'args': {'tag': 'w3'}},
            {'gif': {'name': 's1-a-second', 'preset': 'docs', 'length': 100,
                     'reason': "raw take: a later match's native leave (the window the first match now gets)",
                     'steps': [leave_try('s1', cap=cap)]}}]},
        {'id': 's1-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 8, 'steps': [
            {'run': 'steps.py:after_try', 'on': 'a', 'check': True, 'args': {'tag': 's1'}}]},
        keep['z-a'], keep['z-b'],
    ]
    new[-2]['start'] = 'css-or-searching'
    rows['rows'] = new
    return rows


def main():
    p = argparse.ArgumentParser()
    p.add_argument('src')
    p.add_argument('dst')
    p.add_argument('--cap', type=int, default=80)
    args = p.parse_args()
    src, dst = Path(args.src), Path(args.dst)
    if dst.exists():
        shutil.rmtree(dst)
    dst.mkdir(parents=True)
    (dst / 'rows.json').write_text(json.dumps(build(src, args.cap), indent=2) + '\n')
    shutil.copy2(src / 'steps.py', dst / 'steps.py')
    for sub in ('segments', 'refs'):
        if (src / sub).is_dir():
            shutil.copytree(src / sub, dst / sub)
    print(json.dumps({'dst': str(dst), 'rows': [r['id'] for r in build(src, args.cap)['rows']]}))


if __name__ == '__main__':
    main()
