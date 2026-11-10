"""Rows for the rematch-path observation with the pair split across two hosts (primary 2026-11-10 00:05Z): a on one
host, b on another, one runner per host, synchronised by the other instance's own trace (read over ssh). Both
consoles vanilla; QCF trace points with the 4 Hz menu-gate sampler.

  a (host A)                                        b (host B)
  load, main, Online, Quickplay                      load, main, Online, Quickplay; b readies alone (lobby)
  t1  wait b searching; a readies, joins; plays      t1  plays the match if one starts (throwaway first match)
  m1  wait b searching; a readies, joins; plays;     m1  wait a at the CSS; b readies alone; plays; Rematch? No
      Rematch? Yes (a back in its lobby)
  m2  pause_during_join (from b's arrival in a's     m2  wait a in its re-entered lobby; b readies (joins a);
      room: Plus, frames, Leave Battle if shown)          plays if matched
  m3  wait b at the CSS; a readies alone (lobby      m3  leave its lobby if it waits in one; wait a searching;
      from the CSS); pause_during_join                    b readies (joins a); plays if matched
  z                                                  z

Usage: make_xrm.py <eden/obs01> <dst a> <dst b> --a-host H --a-trace P --b-host H --b-trace P
"""
import argparse
import json
import shutil
from pathlib import Path


def step(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, reason, inner, length=100):
    return {'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': [inner]}}


def only(row, key):
    row = dict(row)
    row['steps'] = [s for s in row['steps'] if s.get('on', key) == key]
    return row


def build(src, key, peer):
    rows = json.loads((src / 'rows.json').read_text())
    keep = {r['id']: r for r in rows['rows']}
    rows['about'] = (f"quickplay-cancel-fix rematch-path observation, the pair split across two hosts: this runner "
                     f"drives instance {key}; {peer['key']} runs on {peer['host']} and is followed through its trace.")
    rows['scope'] = 'QCF rematch-path observation, one instance per host (LAN Online across PCs, TLS on).'
    rows['setup'][0]['ref'] = f'one-instance config: {key} (rc6-bg-qp6 route, vanilla), QCF trace points'
    rows['setup'] = [s for s in rows['setup'][:2]]
    ph, pt = peer['host'], peer['trace']
    new = [only(keep[i], key) for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay')]
    if key == 'a':
        new += [
            {'id': 't1-throwaway', 'start': 'quickplay-menu', 'end': 'css', 'budget_min': 12, 'steps': [
                step('wait_peer', 'a', host=ph, trace=pt, condition='searching', cap_s=420, tag='t1'),
                step('reach_css_tile', 'a'), {'gate': 'css-tile', 'on': 'a'},
                {'play': 'pick', 'on': 'a'}, {'gate': 'ready-banner', 'on': 'a'},
                step('ready_and_wait_session', 'a', tag='t1', cap_s=150),
                step('battle_self', 'a', tag='t1'),
                step('settle', 'a', wait_s=40.0)]},
            {'id': 'm3-before', 'start': 'css', 'end': 'tried', 'budget_min': 10, 'steps': [
                step('wait_peer', 'a', host=ph, trace=pt, condition='at_css', cap_s=300, tag='m3'),
                step('ensure_searching', 'a', tag='m3w'),
                gif('m3-a-css-lobby', "README 'before' take: a's lobby entered from the character select (a hosts), b "
                    "joins from another PC, a pauses during b's loading: no cancel; raw take",
                    step('pause_during_join', 'a', tag='m3', join_cap_s=240, cap_s=80, wait_p2=True, peer_host=ph,
                         peer_trace=pt))]},
            {'id': 'm3-after', 'start': 'tried', 'end': 'a-lobby', 'budget_min': 12, 'steps': [
                step('battle_self', 'a', tag='m3'),
                step('rematch_self', 'a', answer='yes', tag='m3')]},
            {'id': 'm5-native', 'start': 'a-lobby', 'end': 'tried', 'budget_min': 9, 'steps': [
                gif('m5-a-host-rematch-lobby', "native reference: a hosted, answered Rematch Yes; b searches again from "
                    "another PC and joins a's reopened lobby; a pauses during b's loading; raw take",
                    step('pause_during_join', 'a', tag='m5', join_cap_s=240, cap_s=80, wait_p2=True, peer_host=ph,
                         peer_trace=pt))]},
            {'id': 'm5-after', 'start': 'tried', 'end': 'css', 'budget_min': 8, 'steps': [
                step('where_are', 'a', tag='m5-left', wait_s=8.0),
                step('play_if_matched', 'a', tag='m5', wait_s=20),
                step('where_are', 'a', tag='m5-after', wait_s=3.0)]},
        ]
    else:
        new += [
            {'id': 'r01-b-waits', 'start': 'quickplay-menu', 'end': 'b-searching', 'budget_min': 5, 'steps': [
                step('reach_css_tile', 'b'), {'gate': 'css-tile', 'on': 'b'},
                {'play': 'pick', 'on': 'b'}, {'gate': 'ready-banner', 'on': 'b'},
                step('ready_and_wait_searching', 'b', tag='w1')]},
            {'id': 't1-throwaway', 'start': 'b-searching', 'end': 'css', 'budget_min': 14, 'steps': [
                step('play_if_matched', 'b', tag='t1', wait_s=600)]},
            {'id': 'm3-before', 'start': 'css', 'end': 'css', 'budget_min': 16, 'steps': [
                step('wait_peer', 'b', host=ph, trace=pt, condition='searching', cap_s=300, tag='m3'),
                step('ready_until_searching', 'b', tag='m3w'),
                step('play_if_matched', 'b', tag='m3', wait_s=240, answer='no')]},
            {'id': 'm5-native', 'start': 'css', 'end': 'css', 'budget_min': 12, 'steps': [
                step('wait_peer', 'b', host=ph, trace=pt, condition='rematch_lobby', cap_s=300, tag='m5'),
                step('ready_until_searching', 'b', tag='m5w'),
                step('play_if_matched', 'b', tag='m5', wait_s=150),
                step('where_are', 'b', tag='m5-after', wait_s=3.0)]},
        ]
    z = dict(keep[f'z-{key}'])
    z['start'] = 'css'
    new.append(z)
    rows['rows'] = new
    return rows


def main():
    p = argparse.ArgumentParser()
    p.add_argument('src')
    p.add_argument('dst_a')
    p.add_argument('dst_b')
    for k in ('a', 'b'):
        p.add_argument(f'--{k}-host', required=True)
        p.add_argument(f'--{k}-trace', required=True)
    args = p.parse_args()
    src = Path(args.src)
    peers = {'a': {'key': 'b', 'host': args.b_host, 'trace': args.b_trace},
             'b': {'key': 'a', 'host': args.a_host, 'trace': args.a_trace}}
    for key, dst in (('a', Path(args.dst_a)), ('b', Path(args.dst_b))):
        if dst.exists():
            shutil.rmtree(dst)
        dst.mkdir(parents=True)
        rows = build(src, key, peers[key])
        (dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
        shutil.copy2(src / 'steps.py', dst / 'steps.py')
        for sub in ('segments', 'refs'):
            if (src / sub).is_dir():
                shutil.copytree(src / sub, dst / sub)
        print(json.dumps({'dst': str(dst), 'rows': [r['id'] for r in rows['rows']]}))


if __name__ == '__main__':
    main()
