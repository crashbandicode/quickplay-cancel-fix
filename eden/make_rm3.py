"""Pair rows on one host for the rematch rerun on the 0x2C-fixed runtime: q00-q03, r01 (b waits), t1 (throwaway first
match), then srm06's proven flow: m3 a hosts from the CSS and b joins (before take), a Yes / b No, m5 b searches again
and joins a's reopened lobby while a pauses (native reference take), then z.
Usage: make_rm3.py <rows-rm02 folder> <dst folder> [--pilot]
--pilot: a runs the CPU pilot plugin with auto_rematch=on (operator tip 04:03Z), so a's Yes is the game's own answer;
the harness answers only b (No) and then waits for a back in its lobby.
"""
import json
import shutil
import sys
from pathlib import Path

src, dst = Path(sys.argv[1]), Path(sys.argv[2])
pilot = '--pilot' in sys.argv[3:]
if dst.exists():
    shutil.rmtree(dst)
shutil.copytree(src, dst)
rows = json.loads((dst / 'rows.json').read_text())
by = {r['id']: r for r in rows['rows']}


def step(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, reason, inner, length=110):
    return {'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': [inner]}}


new = [by[i] for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay', 'r01-b-waits', 't1-throwaway')]
if pilot:
    # The pilot answers t1's "Rematch?" Yes on a: if t1 ended normally, a waits alone in its reopened lobby, which is
    # the native case right away (skipped if t1 went to the first-of-boot error). A match that starts is played out.
    new += [{'id': 'm0-native', 'start': 'css-or-searching', 'end': 'css-or-searching', 'budget_min': 12, 'steps': [
        step('native_if_lobby', 'a', tag='m0', peer='b', join_cap_s=150, cap_s=80)]}]
new += [
    {'id': 'm3-before', 'start': 'css-or-searching', 'end': 'tried', 'budget_min': 12, 'steps': [
        step('leave_if_searching', 'b', tag='m3b'),
        *([step('leave_if_searching', 'a', tag='m3a')] if pilot else []),
        step('ensure_searching', 'a', tag='m3w'),
        step('ready_until_searching', 'b', tag='m3r'),
        gif('m3-a-css-lobby', "README 'before' take: a's lobby entered from the character select (a hosts), b joins, "
            "a pauses during b's loading: no cancel; raw take",
            step('pause_during_join', 'a', tag='m3', join_cap_s=150, cap_s=80, wait_p2=True))]},
    {'id': 'm3-after', 'start': 'tried', 'end': 'rematch-answered', 'budget_min': 12, 'steps': [
        step('battle', 'a', tag='m3'),
        step('rematch_answers', 'a', tag='m3', answers={'b': 'no'} if pilot else {'a': 'yes', 'b': 'no'}),
        step('wait_where', 'a', tag='m3-rematch', in_lobby=True, cap_s=60.0),
        step('where_are', 'a', tag='m3-rematch', wait_s=3.0)]},
    {'id': 'm5-native', 'start': 'rematch-answered', 'end': 'tried', 'budget_min': 8, 'steps': [
        step('ready_until_searching', 'b', tag='m5b'),
        gif('m5-a-host-rematch-lobby', "native reference: a hosted and answered Rematch Yes; b searches again and "
            "joins a's reopened lobby (0x2C fix); a pauses during b's loading; raw take",
            step('pause_during_join', 'a', tag='m5', join_cap_s=150, cap_s=80, wait_p2=True))]},
    {'id': 'm5-after', 'start': 'tried', 'end': 'css', 'budget_min': 9, 'steps': [
        step('where_are', 'a', tag='m5-left', wait_s=8.0),
        step('after_soft', 'a', tag='m5'),
        step('where_are', 'a', tag='m5-after', wait_s=3.0)]},
    dict(by['z-a'], start='css'), by['z-b'],
]
rows['rows'] = new
rows['about'] = ("quickplay-cancel-fix rematch rerun on the 0x2C-fixed LAN runtime (both consoles vanilla; QCF trace "
                 "points): a throwaway first match; a hosts from the CSS and b joins (before); a Yes / b No; b searches "
                 "again and joins a's reopened lobby while a pauses (native reference).")
if pilot:
    rows['about'] = ("quickplay-cancel-fix rematch-path run on the 0x2C-fixed LAN runtime (QCF trace points; a also runs "
                     "ssbu-cpu-pilot 0.2.2 with auto_rematch=on and P1 the player, so a's Rematch Yes is the game's own "
                     "answer): t1; m0 the native case if a is back in its lobby; m3 before (a hosts from the CSS); a Yes "
                     "/ b No; m5 the native reference.")
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps([r['id'] for r in new]))
