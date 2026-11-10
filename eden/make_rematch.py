"""Rows for the rematch-path observation boot (primary 2026-11-09 21:36Z; both consoles vanilla, QCF trace points):

  t1  a throwaway first match (every boot's first match goes to the error state on this setup, with or without QCF)
  m1  a joins b, both play to the end; at "Rematch?" a answers Yes, b answers No (the operator's case)
  m2  a back in its lobby (re-entered by the rematch Yes); b readies from the character select and joins; during b's
      loading a pauses and takes the cancel back to the character select if it is offered   -> "after" target GIF
  m3  a enters its lobby from the character select, b joins; a pauses during b's loading (no cancel expected)
                                                                                           -> "before" GIF
  z   Unmapped 0, console clean

Usage: make_rematch.py <eden/obs01> <dst folder>
"""
import argparse
import json
import shutil
from pathlib import Path


def step(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, reason, inner, length=90):
    return {'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': [inner]}}


def build(src):
    rows = json.loads((src / 'rows.json').read_text())
    keep = {r['id']: r for r in rows['rows']}
    rows['about'] = ("quickplay-cancel-fix rematch-path observation (both consoles vanilla; detector and tile cursor on "
                     "both; QCF trace points): a throwaway first match (t1); a match where a answers the Rematch? Yes "
                     "and b No (m1); b joins a's re-entered lobby and a pauses during b's loading (m2, the native "
                     "cancel); the same with a's lobby entered from the character select (m3).")
    rows['scope'] = 'QCF rematch-path observation on the Quickplay pair (route from rc6-bg-qp6).'
    rows['setup'][0]['ref'] = ("pair config: instances a and b (rc6-bg-qp6's route): detector + tile cursor only, "
                               "QCF trace points on both")
    new = [
        keep['q00-load'], keep['q01-main'], keep['q02-online'], keep['q03-quickplay'], keep['r01-b-waits'],
        {'id': 't1-throwaway', 'start': 'b-searching', 'end': 'css-or-searching', 'budget_min': 10, 'steps': [
            step('reach_css_tile', 'a'), {'gate': 'css-tile', 'on': 'a'},
            {'play': 'pick', 'on': 'a'}, {'gate': 'ready-banner', 'on': 'a'},
            step('ready_and_wait_session', 'a', tag='t1', cap_s=150),
            step('play_out', 'a', tag='t1')]},
        {'id': 'm1-rematch', 'start': 'css-or-searching', 'end': 'rematch-answered', 'budget_min': 12, 'steps': [
            step('ensure_searching', 'b', tag='m1w'),
            step('css_to_banner', 'a'),
            step('ready_and_wait_session', 'a', tag='m1', cap_s=150),
            step('battle', 'a', tag='m1'),
            step('rematch_answers', 'a', tag='m1', answers={'a': 'yes', 'b': 'no'})]},
        {'id': 'm1-where', 'start': 'rematch-answered', 'end': 'a-lobby', 'budget_min': 3, 'steps': [
            step('where_are', 'a', tag='m1', wait_s=8.0)]},
        {'id': 'm2-native', 'start': 'a-lobby', 'end': 'tried', 'budget_min': 7, 'steps': [
            step('ready_until_searching', 'b', tag='m2b'),
            gif('m2-a-rematch-lobby', "native reference: a's lobby re-entered by Rematch Yes, b joins, a pauses "
                "during b's loading (the operator's working cancel); raw take, trimmed after review",
                step('pause_during_join', 'a', tag='m2', cap_s=75))]},
        {'id': 'm2-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 9, 'steps': [
            step('where_are', 'a', tag='m2-left', wait_s=10.0),
            step('after_soft', 'a', tag='m2'),
            step('where_are', 'a', tag='m2-after', wait_s=5.0)]},
        {'id': 'm3-before', 'start': 'css-or-searching', 'end': 'tried', 'budget_min': 9, 'steps': [
            step('leave_if_searching', 'b', tag='m3b'),
            step('ensure_searching', 'a', tag='m3w'),
            step('ready_until_searching', 'b', tag='m3r'),
            gif('m3-a-css-lobby', "README 'before' take: a's lobby entered from the character select, b joins, a "
                "pauses during b's loading: no cancel; raw take, trimmed after review",
                step('pause_during_join', 'a', tag='m3', cap_s=75))]},
        {'id': 'm3-after', 'start': 'tried', 'end': 'css-or-searching', 'budget_min': 9, 'steps': [
            step('after_soft', 'a', tag='m3')]},
        keep['z-a'], keep['z-b'],
    ]
    new[-2]['start'] = 'css-or-searching'
    rows['rows'] = new
    return rows


def main():
    p = argparse.ArgumentParser()
    p.add_argument('src')
    p.add_argument('dst')
    args = p.parse_args()
    src, dst = Path(args.src), Path(args.dst)
    if dst.exists():
        shutil.rmtree(dst)
    dst.mkdir(parents=True)
    rows = build(src)
    (dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
    shutil.copy2(src / 'steps.py', dst / 'steps.py')
    for sub in ('segments', 'refs'):
        if (src / sub).is_dir():
            shutil.copytree(src / sub, dst / sub)
    print(json.dumps({'dst': str(dst), 'rows': [r['id'] for r in rows['rows']]}))


if __name__ == '__main__':
    main()
