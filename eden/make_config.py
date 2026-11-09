"""Write a QCF pair config from worker B's XPS Quickplay config (rc6-bg-qp6): both instances get only the desync
detector, the tile cursor and (optionally) the QCF NRO; every pin is re-pointed at a copy in the QCF inputs folder;
both instances carry the QCF state-machine trace points.

Usage (on the XPS): make_config.py <rc6-bg-qp6 config.json> <out config.json> <label>
                    [--nro a=<path> --nro b=<path>] [--displays 124,125] [--res <core.resolution_setup>]
"""
import argparse
import hashlib
import json
from pathlib import Path

INPUTS = Path('/home/intpa/eden-runs/inputs-qcf')
OWNER = 'OpenCode tr-opus-qcf'
# Quickplay parent state store, IngameCallSequenceScene state store, PracticeScene result read, scene request.
POINTS = [
    {'target': 'main+0x236b4ac', 'registers': ['x8', 'x9'],
     'captures': [{'expression': 'x20+0x14c', 'type': 'u32'}]},
    {'target': 'main+0x23191bc', 'registers': ['x8', 'x9'],
     'captures': [{'expression': 'x20+0x174', 'type': 'u32'},
                  {'expression': 'x20+0x198', 'type': 'bytes', 'size': 8},
                  {'expression': 'pc+0x2febf34', 'type': 'bytes', 'size': 8}]},
    {'target': 'main+0x23105bc', 'registers': ['x8']},
    {'target': 'main+0x37266d0', 'registers': ['x0', 'x30'],
     'captures': [{'expression': 'x1+8', 'type': 'c-string', 'size': 64},
                  {'expression': 'x2+8', 'type': 'c-string', 'size': 64}]},
    # PracticeScene (the waiting area, vtable 0x50a2e50; context = scene+0x58): state store (x8 new, x9 old) with the
    # match-received word +0xf4 and the online service's round ack mask service+0x26a34 (service =
    # [[0x52c4c58]+8]+0x88, read pc-relative).
    {'target': 'main+0x18514bc', 'registers': ['x8', 'x9'],
     'captures': [{'expression': 'x20+0xf4', 'type': 'u32'},
                  {'expression': '[[[pc+0x3a7379c]+8]+0x88]+0x26a34', 'type': 'u32'}]},
    # PracticeScene message 0xce99fc8cd: "match received" (+0xf4 = 3).
    {'target': 'main+0x185129c',
     'captures': [{'expression': '[[[pc+0x3a739bc]+8]+0x88]+0x26a34', 'type': 'u32'}]},
    # PracticeScene message 0xbe75b6cfc type 6/7: state 3.
    {'target': 'main+0x185113c', 'captures': [{'expression': 'x19+8', 'type': 'u32'}]},
    # PracticeScene state 4 (leave): refused back to state 1 while the round ack mask is non-zero.
    {'target': 'main+0x18518b0', 'rate_limit': 20,
     'captures': [{'expression': 'x0+0x204', 'type': 'u32'},
                  {'expression': '[[[pc+0x3a733a8]+8]+0x88]+0x26a34', 'type': 'u32'}]},
    # PracticeScene quit request (Leave Battle chosen in the waiting menu).
    {'target': 'main+0x1851310', 'registers': ['x30'], 'backtrace': True},
    # Online service round start / ensure-round (the battle-start handshake).
    {'target': 'main+0x16a1df0', 'registers': ['x1', 'x30'], 'backtrace': True,
     'captures': [{'expression': 'x0+0x60', 'type': 'u64'},
                  {'expression': 'x0+0x26a34', 'type': 'u32'},
                  {'expression': 'x0+0x26a62', 'type': 'u8'}]},
    {'target': 'main+0x16b9830', 'registers': ['x30'], 'backtrace': True,
     'captures': [{'expression': 'x0+0x26a34', 'type': 'u32'}, {'expression': 'x0+0x26a62', 'type': 'u8'}]},
    # Round mask writes: the poller's abort/not-ready clears, the service reset, other stores.
    {'target': 'main+0x16a2120'}, {'target': 'main+0x16a21e8'}, {'target': 'main+0x168b500'},
    {'target': 'main+0x16b2d6c'}, {'target': 'main+0x169eadc', 'registers': ['x8', 'x9', 'x24']},
    {'target': 'main+0x1892048', 'registers': ['x8', 'x9']},
    # Session messages at or above 0xbe (round request 0xbe, commit 0xc2, go 0xc3, ...).
    {'target': 'main+0x1685cb0', 'registers': ['x1', 'x3', 'x4', 'x30'], 'condition': 'x1 >= 0xbe',
     'rate_limit': 20},
    {'target': 'main+0x1685ac0', 'registers': ['x1', 'x2', 'x4', 'x30'], 'condition': 'x2 >= 0xbe',
     'rate_limit': 20},
    # The waiting battle's host-side round start (0x1566650 -> 0x16a1df0): x19 = the waiting battle object (its
    # counter +0x180, countdown clock +0x188/+0x190/+0x198, flags +0x1a0), x0 = service (+0x26a40 = countdown ms).
    {'target': 'main+0x1566af4', 'registers': ['x1'],
     'captures': [{'expression': 'x19+0x180', 'type': 'u32'}, {'expression': 'x19+0x188', 'type': 'u64'},
                  {'expression': 'x19+0x190', 'type': 'u64'}, {'expression': 'x19+0x198', 'type': 'bytes',
                                                                 'size': 10},
                  {'expression': 'x0+0x26a40', 'type': 'u32'}]},
    # Same function after its member and ready checks: w20 = the countdown verdict (0x15637a0's result).
    {'target': 'main+0x15669c4', 'registers': ['x20'], 'rate_limit': 10,
     'captures': [{'expression': 'x19+0x180', 'type': 'u32'}]},
    # 0x15637a0's countdown read (every 60 frames): x8 = service, x19 = the waiting battle object.
    {'target': 'main+0x1564088', 'rate_limit': 10,
     'captures': [{'expression': 'x8+0x26a40', 'type': 'u32'}, {'expression': 'x19+0x188', 'type': 'u64'},
                  {'expression': 'x19+0x190', 'type': 'u64'}, {'expression': 'x19+0x198', 'type': 'bytes',
                                                                 'size': 10}]},
    # The waiting battle's states (x0 = the object, +0x1d0 = state function): 0x1563454 leads to the countdown reset
    # 0x1563650 (clock +0x188 = now, +0x190 = 0, flags +0x198/+0x1a0/+0x1a1 = 0), which enters the wait 0x15648d0.
    {'target': 'main+0x1563454', 'registers': ['x0'], 'rate_limit': 5,
     'captures': [{'expression': 'x0+0x188', 'type': 'u64'}, {'expression': 'x0+0x190', 'type': 'u64'}]},
    {'target': 'main+0x1563650', 'registers': ['x0'],
     'captures': [{'expression': 'x0+0x188', 'type': 'u64'}, {'expression': 'x0+0x190', 'type': 'u64'},
                  {'expression': 'x0+0x198', 'type': 'bytes', 'size': 10}]},
    {'target': 'main+0x15648d0', 'registers': ['x0'], 'rate_limit': 5,
     'captures': [{'expression': 'x0+0x188', 'type': 'u64'}, {'expression': 'x0+0x190', 'type': 'u64'},
                  {'expression': 'x0+0x198', 'type': 'bytes', 'size': 10}]},
    # The Quickplay sequence's search start (state 5, first frame): x8 = the service, +0x74 its matchmaking mode
    # before .16b4920 sets it (0 = a new Quickplay session, the QCF plugin's arming condition).
    {'target': 'main+0x2369304', 'registers': ['x0', 'x8'], 'captures': [{'expression': 'x8+0x74', 'type': 'u32'}]},
    # PracticeScene's leave state, first frame (Leave Battle confirmed): the QCF plugin's drop point.
    {'target': 'main+0x1851930', 'registers': ['x19']},
    # The local member state setter (svc, state): what this console reports to the room (the host starts the round
    # once every member reports 7), with the caller.
    {'target': 'main+0x168db60', 'registers': ['x1', 'x30'], 'backtrace': True, 'rate_limit': 20},
    # 0x1566650's host branch (this station is the session host): sampled to tell host from joiner.
    {'target': 'main+0x1566758', 'rate_limit': 1, 'captures': [{'expression': 'x19+0x180', 'type': 'u32'}]},
]


def sha256(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def pin(path):
    path = Path(path)
    return {'path': str(path), 'sha256': sha256(path)}


def main():
    p = argparse.ArgumentParser()
    p.add_argument('source')
    p.add_argument('out')
    p.add_argument('label')
    p.add_argument('--nro', action='append', default=[], help='a=<path> or b=<path>: the QCF NRO for that instance')
    p.add_argument('--displays', default='124,125')
    p.add_argument('--res', type=int, help='core.resolution_setup for both instances (XPS default is 1 = 0.5x)')
    p.add_argument('--inputs', default=str(INPUTS), help='folder with the detector and tile cursor NROs')
    p.add_argument('--overlay', action='append', default=[],
                   help='<file>@<profile-relative path>: copied into both instances\' profiles (pinned)')
    p.add_argument('--host-base', help='JSON with top-level keys to override (host, hostname, runner, ...)')
    p.add_argument('--authorization', required=True)
    args = p.parse_args()
    src = json.loads(Path(args.source).read_text())
    nros = dict(item.split('=', 1) for item in args.nro)
    displays = args.displays.split(',')
    detector = Path(args.inputs) / 'libssbu_desync_detector.nro'
    tile = Path(args.inputs) / 'libssbu_tile_cursor.nro'
    overlays = [dict(pin(item.split('@', 1)[0]), to=item.split('@', 1)[1]) for item in args.overlay]
    cfg = {k: v for k, v in src.items() if k not in ('final', 'rows', 'instances')}
    instance_base = {}
    if args.host_base:
        base = json.loads(Path(args.host_base).read_text())
        instance_base = base.pop('instance', {})
        cfg.update(base)
    cfg.update(owner=OWNER, label=args.label, authorization=args.authorization, runtime=5400,
               work_root=f'/home/intpa/eden-runs/{args.label}/work', state_dir=f'/home/intpa/eden-runs/{args.label}/state')
    cfg['instances'] = []
    for i, item in enumerate(src['instances'][:2]):
        key = item['key']
        inst = {k: v for k, v in item.items() if k not in ('plugins', 'files', 'trace', 'sources')}
        inst.update(instance_base)
        inst['display'] = ':' + displays[i]
        inst['plugins'] = [pin(detector), pin(tile)] + ([pin(nros[key])] if key in nros else [])
        inst['files'] = [dict(o) for o in overlays]
        inst['sources'] = {'detector_log': 'user/sdmc/ultimate/ssbu-desync-detector/events.log',
                           'tile_log': 'user/sdmc/ultimate/ssbu-tile-cursor/events.log',
                           'qcf_log': 'user/sdmc/ultimate/quickplay-cancel-fix/events.log'}
        inst['trace'] = {'unmapped_access': 'log', 'symbols': [], 'points': POINTS}
        if args.res is not None:
            inst['eden_settings'] = {'core.resolution_setup': args.res}
        cfg['instances'].append(inst)
    Path(args.out).write_text(json.dumps(cfg, indent=2) + '\n')
    print(json.dumps({'out': args.out, 'plugins': {i['key']: [Path(x['path']).name for x in i['plugins']]
                                                   for i in cfg['instances']}}))


if __name__ == '__main__':
    main()
