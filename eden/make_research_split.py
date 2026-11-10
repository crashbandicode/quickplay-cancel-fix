"""The 14:55Z research rows (make_research.py) for a pair split across two hosts (the operator's 15:33Z rule:
performance-sensitive runs on the Yoga, the Ally or the XPS only): a on one host, b on another, one runner each,
synchronised through the other instance's own trace over ssh. The CPU pilot on both answers every Rematch? Yes.

  a (host A)                                          b (host B)
  load, main, Online, Quickplay                        load, main, Online, Quickplay; r01: b readies alone (hosts)
  m1   wait b searching; a picks, readies, joins;      m1   plays the match a's join starts
       plays its own fighter
  ry1  wait own answer; 25 s GIF + frames of a's       ry1  wait own answer; B at each phase of the re-entry;
       screen; plays the rematch match if one starts        plays the match if one starts
  ry2  same (b presses Plus, Leave Battle if shown)    ry2  Plus at each phase, Leave Battle taken if shown
  ry3  only if ry2 ended in a match (no GIF)           ry3  same, only if ry2 ended in a match
  m2   leave its lobby; wait b searching; a joins      m2   wait a at the CSS; b readies (hosts); plays
       from the CSS; plays
  ry4/ry5 as ry2/ry3                                   ry4/ry5 as ry2/ry3
  z                                                    z
Usage: make_research_split.py <eden/obs01> <dst a> <dst b> --a-host H --a-trace P --b-host H --b-trace P
"""
import argparse
import json
import shutil
from pathlib import Path


def step(run, on, **args):
    return {'run': f'steps.py:{run}', 'on': on, 'check': True, 'args': args}


def gif(name, on, length, reason, inner):
    return {'on': on, 'gif': {'name': name, 'preset': 'docs', 'length': length, 'reason': reason, 'steps': inner}}


def only(row, key):
    row = dict(row)
    row['steps'] = [s for s in row['steps'] if s.get('on', key) == key]
    return row


def a_try(tag, button, take_gif=True, skip=None):
    s = {'skip_if_left': skip} if skip else {}
    watch = [step('watch_reentry', 'a', tag=tag, cap_s=25, **s)]
    body = ([gif(f'{tag}-a-rematch', 'a', 25, f"I2 evidence: a's screen from its Rematch answer while b (another PC) "
                 f"presses {button} at each phase of the re-entry; capped at 25 s so the match that may follow isn't "
                 "left idle long during the encode; raw take", watch)] if take_gif else watch)
    return [step('wait_answered', 'a', tag=tag, cap_s=120, **s)] + body + [
        step('battle_self_if', 'a', tag=tag, wait_s=60, **s), step('where_are', 'a', tag=f'{tag}-after', wait_s=3.0)]


def b_try(tag, button, ph, pt, skip=None, leave=True):
    s = {'skip_if_left': skip} if skip else {}
    return [step('wait_answered', 'b', tag=tag, cap_s=120, **s),
            step('rematch_tries', 'b', tag=tag, button=button, cap_s=90, leave=leave, peer_host=ph, peer_trace=pt, **s),
            step('battle_self_if', 'b', tag=tag, wait_s=10, **s),
            step('where_are', 'b', tag=f'{tag}-after', wait_s=3.0)]


def build(src, key, peer):
    rows = json.loads((src / 'rows.json').read_text())
    keep = {r['id']: r for r in rows['rows']}
    rows['about'] = (f"QCF research (primary 14:55Z, I1/I2) split across two hosts: this runner drives instance {key}; "
                     f"{peer['key']} runs on {peer['host']} and is followed through its trace. The CPU pilot on both "
                     "answers every Rematch? Yes; the research LAN runtime logs each RMC request body.")
    rows['scope'] = 'QCF research I1/I2, one instance per host (LAN Online across PCs, TLS on).'
    rows['setup'][0]['ref'] = f'one-instance config: {key}, QCF trace points'
    rows['setup'] = [s for s in rows['setup'][:2]]
    ph, pt = peer['host'], peer['trace']
    new = [only(keep[i], key) for i in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay')]
    tries = [('ry1', 'b', True, None), ('ry2', 'plus', True, 'ry1'), ('ry3', 'plus', False, 'ry2')]
    tries2 = [('ry4', 'plus', True, None), ('ry5', 'plus', False, 'ry4')]
    if key == 'a':
        new.append({'id': 'm1', 'start': 'quickplay-menu', 'end': 'any', 'budget_min': 14, 'steps': [
            step('wait_peer', 'a', host=ph, trace=pt, condition='searching', cap_s=420, tag='m1'),
            step('reach_css_tile', 'a'), {'gate': 'css-tile', 'on': 'a'},
            {'play': 'pick', 'on': 'a'}, {'gate': 'ready-banner', 'on': 'a'},
            step('ready_and_wait_session', 'a', tag='m1', cap_s=150),
            step('battle_self', 'a', tag='m1')]})
        for tag, button, g, skip in tries:
            new.append({'id': tag, 'start': 'any', 'end': 'any', 'budget_min': 10, 'steps': a_try(tag, button, g, skip)})
        new.append({'id': 'm2', 'start': 'any', 'end': 'any', 'budget_min': 14, 'steps': [
            step('leave_if_searching', 'a', tag='m2a'),
            step('wait_peer', 'a', host=ph, trace=pt, condition='searching', cap_s=300, tag='m2'),
            step('css_to_banner', 'a'),
            step('ready_and_wait_session', 'a', tag='m2', cap_s=150),
            step('battle_self', 'a', tag='m2')]})
        for tag, button, g, skip in tries2:
            new.append({'id': tag, 'start': 'any', 'end': 'any', 'budget_min': 10, 'steps': a_try(tag, button, g, skip)})
    else:
        new.append({'id': 'r01-b-waits', 'start': 'quickplay-menu', 'end': 'b-searching', 'budget_min': 5, 'steps': [
            step('reach_css_tile', 'b'), {'gate': 'css-tile', 'on': 'b'},
            {'play': 'pick', 'on': 'b'}, {'gate': 'ready-banner', 'on': 'b'},
            step('ready_and_wait_searching', 'b', tag='w1')]})
        new.append({'id': 'm1', 'start': 'b-searching', 'end': 'any', 'budget_min': 14, 'steps': [
            step('battle_self_if', 'b', tag='m1', wait_s=600)]})
        for tag, button, g, skip in tries:
            new.append({'id': tag, 'start': 'any', 'end': 'any', 'budget_min': 10,
                        'steps': b_try(tag, button, ph, pt, skip, leave=(button == 'plus'))})
        new.append({'id': 'm2', 'start': 'any', 'end': 'any', 'budget_min': 14, 'steps': [
            step('leave_if_searching', 'b', tag='m2b'),
            step('wait_peer', 'b', host=ph, trace=pt, condition='at_css', cap_s=300, tag='m2'),
            step('ensure_searching', 'b', tag='m2w'),
            step('battle_self_if', 'b', tag='m2', wait_s=300)]})
        for tag, button, g, skip in tries2:
            new.append({'id': tag, 'start': 'any', 'end': 'any', 'budget_min': 10,
                        'steps': b_try(tag, button, ph, pt, skip)})
    z = dict(keep[f'z-{key}'])
    z['start'] = 'any'
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
