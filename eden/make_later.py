"""Rows for the later-match boot: make_rm3.py --pilot's flow with QCF 0.2.0 (release) and the pilot both on a.
  m0  after t1 a's pilot answers Rematch Yes; b joins a's reopened lobby; a pauses during b's load: the game's own
      leave, with the plugin installed (the lobby's own count is 2, which the plugin leaves alone).
  m3  a hosts from the CSS (the plugin's case), b joins; a pauses during b's load: Leave Battle expected; no leave,
      the match is played out (no GIF around a match start: the encode left the fighters idle in qcf-xrm07/08).
  m5  a's pilot answers Rematch Yes after m3; b joins a's reopened lobby; a pauses during b's load: the game's own
      leave again, now after a played match that started in a character-select lobby with the plugin.
Usage: make_later.py <rows-rm06 folder (make_rm3.py --pilot)> <dst folder>
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


def unwrap(steps):
    out = []
    for s in steps:
        if 'gif' in s:
            out.extend(s['gif']['steps'])
        else:
            out.append(s)
    return out


for row in rows['rows']:
    row['steps'] = unwrap(row['steps'])
    for s in row['steps']:
        if s.get('run') == 'steps.py:pause_during_join' and s['args'].get('tag') == 'm3':
            s['args']['leave'] = False


def step(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, on, length, reason, inner):
    """A README take recorded on `on` (a gif step without `on` records on the runner's default, a: qcf-xrm07/08's
    'before' takes showed a's screen). A match that starts inside a take waits for the encode with idle fighters,
    so takes a match follows are capped short (Eden stops recording at `length`)."""
    return {'on': on, 'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': inner}}


by = {r['id']: r for r in rows['rows']}
tail = [by.pop('z-a'), by.pop('z-b')]
rows['rows'] = [r for r in rows['rows'] if r['id'] not in ('z-a', 'z-b')]
rows['rows'] += [
    {'id': 'gif-before', 'start': 'css', 'end': 'css', 'budget_min': 16, 'steps': [
        step('leave_if_searching', 'a', tag='gba'),
        step('leave_if_searching', 'b', tag='gbb'),
        gif('before-css-ready', 'b', 40, "README 'before', part 1: vanilla console b readies from the character select "
            "and enters its Quickplay lobby; raw take", [step('ensure_searching', 'b', tag='gbw')]),
        step('ready_until_searching', 'a', tag='gbr'),
        gif('before-lobby', 'b', 18, "README 'before', part 2: a joins b's lobby; b pauses during a's loading: no "
            "Leave Battle; capped so the match that follows isn't left idle during the encode; raw take",
            [step('pause_during_join', 'b', tag='gb', join_cap_s=150, cap_s=60, wait_p2=True, leave=False)]),
        step('after_soft', 'a', tag='gb'),
        step('where_are', 'a', tag='gb-after', wait_s=3.0)]},
    {'id': 'gif-after', 'start': 'css', 'end': 'css', 'budget_min': 14, 'steps': [
        step('leave_if_searching', 'a', tag='gaa'),
        step('leave_if_searching', 'b', tag='gab'),
        gif('after-css-ready', 'a', 40, "README 'after', part 1: console a (QCF plugin) readies from the character "
            "select and enters its Quickplay lobby; raw take", [step('ensure_searching', 'a', tag='gaw')]),
        step('ready_until_searching', 'b', tag='gar'),
        gif('after-lobby', 'a', 80, "README 'after', part 2: b joins a's lobby; a pauses during b's loading: Leave "
            "Battle, Yes, and the load back to the character select (no match follows); raw take",
            [step('pause_during_join', 'a', tag='ga', join_cap_s=150, cap_s=60, wait_p2=True, leave=True),
             step('where_are', 'a', tag='ga-css', wait_s=45.0)]),
        step('where_are', 'a', tag='ga-b-waits', wait_s=1.0),
        step('leave_if_searching', 'b', tag='gabl'),
        step('where_are', 'a', tag='ga-after', wait_s=3.0)]},
] + tail
ids = [r['id'] for r in rows['rows']]
assert ids[:6] == ['q00-load', 'q01-main', 'q02-online', 'q03-quickplay', 'r01-b-waits', 't1-throwaway'], ids
assert 'm0-native' in ids and 'm5-native' in ids, ids
rows['about'] = ("quickplay-cancel-fix 0.2.0 later-match boot on the 0x2C-fixed LAN runtime: a runs the plugin (release "
                 "build) and ssbu-cpu-pilot 0.2.2 (auto_rematch=on, P1 the player) so its Rematch Yes is the game's own; "
                 "b is vanilla. m0 and m5: the rematch lobby's own leave with the plugin installed; m3: a character-select "
                 "lobby with the plugin (tab expected), the match played out.")
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps(ids))
