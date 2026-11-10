"""Rows for the 0.3.0 hold boot (primary 18:17Z/20:48Z): a (QCF 0.3.0) joins vanilla b's room; the hold, the
leave during it, held matches played out, a rematch after a held match, and the native "before".

  r01   b searches alone (a fresh room).
  h3l   hold 3 s; a joins from the CSS, opens the menu during the hold and takes Leave Battle (README "after" take).
  h3    b searches anew (a fresh countdown); a joins, peeks at the menu (B closes it); the hold runs out, the match is
        played to the end.
  ry3   both answer Rematch Yes (the CPU pilot); a re-enters as the joiner and holds again; the rematch match is
        played to the end (no freeze after a held match).
  ry3l  Rematch Yes again; a takes Leave Battle during the rematch's hold; b keeps searching.
  h6p   hold 6 s, b past its countdown (45 s in its room); a joins, peeks; the match is played; ry6l as ry3l.
  h10   hold 10 s, a fresh countdown; h10p hold 10 s, b past its countdown; each played, each followed by a rematch
        leave (ry10l, ry10pl).
  h0    hold 0 (off, 0.2.1's behaviour): a joins; Plus as it comes (README "before" take, capped short because a match
        follows); the match is played.
Every hold row records a's member reports (the held 6, the plugin's 7), b's countdown/ready/round records, frames of
both about every second, and what it expected (held, h0 not; the outcome; the hold's length); v-hold fails the boot
if any row saw otherwise (after the z rows, so a mismatch stops nothing).
Usage: make_hold.py <rows-fz02 folder> <steps.py> <dst folder>
"""
import json
import shutil
import sys
from pathlib import Path

src, steps_py, dst = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
if dst.exists():
    shutil.rmtree(dst)
shutil.copytree(src, dst)
(dst / 'gdbdump.py').unlink(missing_ok=True)
shutil.copy2(steps_py, dst / 'steps.py')
rows = json.loads((dst / 'rows.json').read_text())
by = {r['id']: r for r in rows['rows']}
load = dict(by['q00-load'])
load['steps'] = [s for s in load['steps'] if s.get('run') not in ('steps.py:gdb_on', 'steps.py:gdb_attached')]


def st(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, on, length, reason, inner):
    return {'on': on, 'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': inner}}


def fresh_b(tag):
    """b leaves its room and searches anew, so a joins a room whose countdown has just started."""
    return [st('leave_waiting', 'b', tag=f'{tag}-bl'), st('css_to_banner', 'b'),
            st('ready_and_wait_searching', 'b', tag=f'{tag}-bw')]


def held_match(tag, seconds, peer_waited_s=0, fresh=False, config=True):
    steps = ([st('set_hold', 'a', seconds=seconds)] if config else []) + (fresh_b(tag) if fresh else [])
    steps += [st('css_to_banner', 'a'),
              st('hold_join', 'a', tag=tag, leave=False, peer_waited_s=peer_waited_s),
              st('expect_hold', 'a', tag=tag, held=True, menu=True, tab=True, name=True, outcome='matched',
                 min_hold_s=round(seconds * 0.9, 2)),
              st('battle', 'a', tag=tag)]
    return {'id': tag, 'start': 'b-searching', 'end': 'results', 'budget_min': 10, 'steps': steps}


def rematch_leave(tag):
    return {'id': tag, 'start': 'results', 'end': 'b-searching', 'budget_min': 6, 'steps': [
        st('wait_answered', 'a', tag=tag, cap_s=120),
        st('hold_rematch', 'a', tag=tag, leave=True),
        st('expect_hold', 'a', tag=tag, held=True, menu=True, tab=True, name=True, outcome='left'),
        st('where_are', 'a', tag=f'{tag}-after', wait_s=6.0),
        st('ensure_searching', 'b', tag=f'{tag}-bw')]}


r01 = {'id': 'r01-b-waits', 'start': 'quickplay-menu', 'end': 'b-searching', 'budget_min': 4, 'steps': [
    st('reach_css_tile', 'b'), {'gate': 'css-tile', 'on': 'b'}, {'play': 'pick', 'on': 'b'},
    {'gate': 'ready-banner', 'on': 'b'}, st('ready_and_wait_searching', 'b', tag='w1')]}
new = [load] + [by[i] for i in ('q01-main', 'q02-online', 'q03-quickplay')] + [r01]
new += [
    {'id': 'h3l', 'start': 'b-searching', 'end': 'b-searching', 'budget_min': 8, 'steps': [
        st('set_hold', 'a', seconds=3),
        st('reach_css_tile', 'a'), {'gate': 'css-tile', 'on': 'a'}, {'play': 'pick', 'on': 'a'},
        {'gate': 'ready-banner', 'on': 'a'},
        gif('after-joiner-hold', 'a', 45, "README 'after' (0.3.0): console a (QCF 0.3.0, hold 3 s) readies from the "
            "character select and joins b's room; during the hold Pause has Leave Battle; Yes, and back to the "
            "character select (no match follows); raw take",
            [st('hold_join', 'a', tag='h3l', leave=True)]),
        st('expect_hold', 'a', tag='h3l', held=True, menu=True, tab=True, name=True, outcome='left'),
        st('where_are', 'a', tag='h3l-after', wait_s=6.0),
        st('ensure_searching', 'b', tag='h3l-bw')]},
    held_match('h3', 3, fresh=True, config=False),
    {'id': 'ry3', 'start': 'results', 'end': 'results', 'budget_min': 9, 'steps': [
        st('wait_answered', 'a', tag='ry3', cap_s=120),
        st('hold_rematch', 'a', tag='ry3', leave=False),
        st('expect_hold', 'a', tag='ry3', held=True, menu=True, tab=True, name=True, outcome='matched',
           min_hold_s=2.7),
        st('battle', 'a', tag='ry3')]},
    rematch_leave('ry3l'),
    held_match('h6p', 6, peer_waited_s=45),
    rematch_leave('ry6l'),
    held_match('h10', 10, fresh=True),
    rematch_leave('ry10l'),
    held_match('h10p', 10, peer_waited_s=45, config=False),
    rematch_leave('ry10pl'),
    {'id': 'h0', 'start': 'b-searching', 'end': 'results', 'budget_min': 10, 'steps': [
        st('set_hold', 'a', seconds=0)] + fresh_b('h0') + [
        st('css_to_banner', 'a'),
        gif('before-joiner-native', 'a', 18, "README 'before' (0.3.0): the same join with the hold off (0.2.1): a "
            "readies and joins b's room, Pause as it comes; capped so the match that follows isn't left idle long "
            "during the encode; raw take",
            [st('hold_join', 'a', tag='h0', leave=True, cap_s=45, expect_held=False)]),
        st('expect_hold', 'a', tag='h0', held=False, menu=None, tab=None),
        st('battle_if_matched', 'a', tag='h0')]},
    dict(by['z-a'], start='results'), by['z-b'],
    {'id': 'v-hold', 'start': 'css', 'end': 'css', 'budget_min': 1, 'steps': [st('hold_verdict', 'a')]},
]
rows['rows'] = new
rows['about'] = ("QCF 0.3.0, the joiner's ready hold (primary 18:17Z/20:48Z): a (QCF 0.3.0 release) joins vanilla "
                 "b's room with holds of 3, 6 and 10 s (fresh and past countdowns), leaves during holds on first "
                 "matches and rematches, plays held matches and a rematch after one; h0 is the hold off. b traced; "
                 "the research LAN runtime.")
(dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
print(json.dumps([r['id'] for r in new]))
