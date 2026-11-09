"""Rows for an "after" boot on a Quickplay pair (a with the QCF NRO, b vanilla), from the obs01 rows' gates,
segments and hooks (primary 2026-11-09 18:40Z; revised after qcf-xafter02):

  f1  a's first match of a Quickplay session, joining b whose countdown has run out: a opens the waiting menu
      and leaves in the hold                                                -> the README "after" GIF take
  s1  b leaves its waiting area and searches again (a fresh room), a readies at once and tries the leave:
      the next match, the leave-menu timing (native unless the game starts a new session after a leave)
  s2  the next match without a leave, played to its end on both (detector sync_end, desync_seen=0)
  z   Unmapped 0, console clean, QCF events

Outcomes are recorded, not expected (a failed expect stops the runner with a match running); the verdicts come
from the trace afterwards (tools/verdict.py).

Usage: make_safter.py <eden/obs01> <dst folder> [--cap 90]
"""
import argparse
import json
import shutil
from pathlib import Path

INSTALLED = r'plugin=quickplay-cancel-fix/\S+ build=diag [^\n]*result=installed heap_allocations=0'


def leave_try(tag, host_wait_s=None, try_leave=True, cap=90):
    args = {'tag': tag, 'cap_s': cap, 'try_leave': try_leave}
    if host_wait_s is not None:
        args['host_wait_s'] = host_wait_s
    return {'run': 'steps.py:ready_and_try_leave', 'on': 'a', 'check': True, 'args': args}


def gif(name, reason, step):
    return {'gif': {'name': name, 'preset': 'docs', 'length': 110, 'reason': reason, 'steps': [step]}}


def build(src, cap):
    rows = json.loads((src / 'rows.json').read_text())
    keep = {r['id']: r for r in rows['rows']}
    rows['about'] = ("quickplay-cancel-fix after boot (a with the QCF diag NRO, b vanilla; detector and tile cursor on "
                     "both): a's first match of a Quickplay session as the joiner of b's room after b's countdown ran "
                     "out, left in the hold (f1); the next match with b's room fresh and a leave attempt (s1, the "
                     "leave-menu timing); the next match played to its end (s2). Trace points on both.")
    rows['scope'] = 'QCF after rows on the Quickplay pair (route from rc6-bg-qp6).'
    rows['setup'][0]['ref'] = ("pair config: instances a and b (rc6-bg-qp6's route): detector + tile cursor, the QCF "
                               "diag NRO on a")
    rows['gates']['qcf-installed'] = {'kind': 'log', 'source': 'qcf_log', 'whole_file': True, 'cap': 120,
                                      'pattern': INSTALLED}
    load = keep['q00-load']
    at = max(i for i, s in enumerate(load['steps']) if s.get('gate') == 'installed') + 1
    load['steps'][at:at] = [{'gate': 'qcf-installed', 'on': 'a'}]
    after = "raw take for the README 'after' GIF; trimmed under 10 s after review"
    new = [
        keep['q00-load'], keep['q01-main'], keep['q02-online'], keep['q03-quickplay'], keep['r01-b-waits'],
        {'id': 'f1-first-leave', 'start': 'b-searching', 'end': 'tried', 'budget_min': 6, 'steps': [
            {'run': 'steps.py:reach_css_tile', 'on': 'a', 'check': True}, {'gate': 'css-tile', 'on': 'a'},
            {'play': 'pick', 'on': 'a'}, {'gate': 'ready-banner', 'on': 'a'},
            gif('f1-a-first-after', after, leave_try('f1', host_wait_s=30, cap=cap))]},
        {'id': 'f1-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 8, 'steps': [
            {'run': 'steps.py:after_try', 'on': 'a', 'check': True, 'args': {'tag': 'f1'}}]},
        {'id': 's1-fresh-leave', 'start': 'css-or-searching', 'end': 'tried', 'budget_min': 8, 'steps': [
            {'run': 'steps.py:leave_waiting', 'on': 'b', 'check': True, 'args': {'tag': 'b-lw'}},
            {'run': 'steps.py:css_to_banner', 'on': 'a', 'check': True},
            {'run': 'steps.py:ensure_searching', 'on': 'b', 'check': True, 'args': {'tag': 'w2'}},
            gif('s1-a-next', "raw take: the next match's leave (the native window, for comparison)",
                leave_try('s1', cap=cap))]},
        {'id': 's1-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 8, 'steps': [
            {'run': 'steps.py:after_try', 'on': 'a', 'check': True, 'args': {'tag': 's1'}}]},
        {'id': 's2-next-play', 'start': 'css-or-searching', 'end': 'tried', 'budget_min': 7, 'steps': [
            {'run': 'steps.py:css_to_banner', 'on': 'a', 'check': True},
            {'run': 'steps.py:ensure_searching', 'on': 'b', 'check': True, 'args': {'tag': 'w3'}},
            leave_try('s2', try_leave=False, cap=cap)]},
        {'id': 's2-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 8, 'steps': [
            {'run': 'steps.py:after_try', 'on': 'a', 'check': True, 'args': {'tag': 's2'}}]},
        keep['z-a'], keep['z-b'],
    ]
    new[-2]['start'] = 'css-or-searching'
    rows['rows'] = new
    return rows


def main():
    p = argparse.ArgumentParser()
    p.add_argument('src')
    p.add_argument('dst')
    p.add_argument('--cap', type=int, default=90)
    args = p.parse_args()
    src, dst = Path(args.src), Path(args.dst)
    if dst.exists():
        shutil.rmtree(dst)
    dst.mkdir(parents=True)
    rows = build(src, args.cap)
    (dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
    shutil.copy2(src / 'steps.py', dst / 'steps.py')
    for sub in ('segments', 'refs'):
        if (src / sub).is_dir():
            shutil.copytree(src / sub, dst / sub)
    print(json.dumps({'dst': str(dst), 'rows': [r['id'] for r in rows['rows']]}))


if __name__ == '__main__':
    main()
