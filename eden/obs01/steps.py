"""quickplay-cancel-fix Eden step hooks (tools/eden-runner StepContext).

Two vanilla-or-plugin instances on the XPS Quickplay route (a and b pair only with each other: rating runtime, TLS).
b searches first and waits in Quickplay's waiting area; a readies and is matched at once. `ready_and_try_leave` is the
row under test: a readies, then tries the game's own cancel (Plus opens the waiting menu, Up to "Leave Battle", A,
Yes) until a match session starts or the leave goes through. Every look is kept as a frame; the trace stream of the
Quickplay state machines (trace points in config.json) is copied into the row folder.

Route pieces come from rc6-bg-qp6 (worker B's XPS Quickplay rows): to_main (online_steps.py), reach_css_tile
(xps_steps.py), the READY TO FIGHT look (qp.py ready_plus), _vary_still and keepalive (of_steps.py).
"""
import json
import os
import re
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REFS = HERE / 'refs'
MAIN_REF = REFS / 'main.png'
BANNER_ROI = [200, 110, 600, 240]       # READY TO FIGHT banner (ready-banner gate)
TAB_BOX = [45, 4, 295, 58]              # the waiting menu's "Leave Battle" tab, 1280x674 frame
DIALOG_ROI = [440, 285, 840, 325]       # a centered two-button dialog's text line (Rematch?, Leave?)
SESSION = re.compile(r'session slot=\d+ kind=online')
SYNC_END = re.compile(r'sync_end trigger=\w+[^\n]*')
# config.json trace points (both instances): the Quickplay parent's and IngameCallSequenceScene's state stores, the
# PracticeScene result read and the scene request.
POINTS = {'main+0x236b4ac': 'parent', 'main+0x23191bc': 'child', 'main+0x23105bc': 'practice_result',
          'main+0x37266d0': 'scene', 'main+0x18514bc': 'practice_state', 'main+0x185129c': 'match_received',
          'main+0x185113c': 'practice_msg67', 'main+0x18518b0': 'practice_leave', 'main+0x1851310': 'quit_request',
          'main+0x16a1df0': 'round_start', 'main+0x16b9830': 'round_ensure', 'main+0x16a2120': 'mask_abort',
          'main+0x16a21e8': 'mask_notready', 'main+0x168b500': 'mask_reset', 'main+0x16b2d6c': 'mask_store_b2d6c',
          'main+0x169eadc': 'mask_store_9eadc', 'main+0x1892048': 'mask_store_92048', 'main+0x1685cb0': 'send',
          'main+0x1685ac0': 'send_to', 'main+0x1566af4': 'host_round_start', 'main+0x15669c4': 'host_ready_check',
          'main+0x1564088': 'countdown_read'}


def _size(path):
    try:
        return os.path.getsize(path)
    except OSError:
        return 0


def _tail(path, start):
    try:
        with open(path, 'rb') as f:
            f.seek(start)
            return f.read().decode(errors='replace')
    except OSError:
        return ''


def _trace_path(ctx, on=None):
    return ctx.runner.instances[on or ctx.on].get('trace_jsonl')


def _save(ctx, name, data):
    path = Path(ctx.row_dir) / f'{name}.json'
    path.write_text(json.dumps(data, indent=1) + '\n')
    ctx.log({name: data if len(json.dumps(data)) < 2000 else str(path)})
    return data


def _diff(ctx, shot, ref, roi):
    return ctx.runner.roi_diff(str(shot), str(ref), list(roi))


def _trace_records(path, start):
    out = []
    for line in _tail(path, start).splitlines():
        try:
            out.append(json.loads(line))
        except ValueError:
            continue
    return out


def _kind(record):
    for loc in (record.get('locations') or {}).values():
        if loc in POINTS:
            return POINTS[loc]
    return None


def _states(records):
    """The QCF trace points' records, compact: kind, frame, x8 (new state / result), x9 (old state), captures."""
    rows = []
    for r in records:
        kind = _kind(r)
        if kind is None or r.get('event') != 'entry':
            continue
        regs = r.get('registers') or {}
        caps = r.get('captures') or []
        rows.append({'kind': kind, 'frame': r.get('frame'), 'x8': regs.get('x8'), 'x9': regs.get('x9'),
                     'regs': regs, 'caps': [c.get('value') if c.get('type') == 'c-string' else
                                            (c.get('hex') or c.get('value')) for c in caps]})
    return rows


# --- navigation -------------------------------------------------------------------------------------------------------

def to_main(ctx, roi=(150, 265, 450, 335), tolerance=8, tries=6, wait_s=4.0):
    """Opening movie, attract demo or title -> the main menu: look; if it isn't the main menu, Plus, look again."""
    diffs = []
    for attempt in range(tries + 1):
        path = ctx.shot(f'to-main-{attempt}')
        diff = ctx.runner.roi_diff(path, str(MAIN_REF), list(roi), True)
        diffs.append(round(diff, 1))
        if diff <= tolerance:
            return _save(ctx, f'to-main-{ctx.on}', {'attempt': attempt, 'diffs': diffs})
        if attempt < tries:
            ctx.press('plus', 150)
            time.sleep(wait_s)
    raise RuntimeError(f'main menu not reached after {tries} presses; diffs {diffs}')


def reach_css_tile(ctx, tries=5, wait_s=4.0):
    """Quickplay menu or its character select -> a token on a CSS tile (`settled screen=css at=N hovered=N`)."""
    runner, on = ctx.runner, ctx.on
    tile_log = runner.instances[on]['tile_log']
    start = _size(tile_log)
    shot = runner.shot(on, 'css-look')
    if _diff(ctx, shot, REFS / 'quickplay-menu.png', [40, 5, 300, 45]) <= 8:
        ctx.play('to-css')
    for i in range(tries):
        ctx.check_stop()
        time.sleep(wait_s)
        if re.search(r'settled screen=css at=(\d+) hovered=\1\b', _tail(tile_log, start)):
            return {'tile_steps': i}
        ctx.play('tile-up')
    raise RuntimeError(f'no settled css tile on {on} after {tries} tile steps')


def css_to_banner(ctx, cap_s=20):
    """Character select reached after a match or a leave: the READY TO FIGHT banner up (token placed), or a tile step
    and A to place the token. Looks first."""
    shot = ctx.shot('css-look')
    diff = _diff(ctx, shot, REFS / 'ready-banner.png', BANNER_ROI)
    if diff <= 40:
        return {'banner': 'already up', 'diff': round(diff, 2)}
    ctx.play('tile-up')
    ctx.gate('css-tile')
    ctx.play('pick')
    ctx.gate('ready-banner-any')
    return {'banner': 'stepped and placed', 'diff_before': round(diff, 2)}


def _ready(ctx, tag, tries=4):
    """READY TO FIGHT up -> Plus starts the search (qp.py ready_plus); no banner -> A places the token, look again."""
    looks = []
    for i in range(tries):
        shot = ctx.shot(f'{tag}-ready-look-{i}')
        diff = _diff(ctx, shot, REFS / 'ready-banner.png', BANNER_ROI)
        looks.append(round(diff, 2))
        if diff <= 40:
            ctx.press('plus', 120)
            return {'looks': looks, 'ready_at': time.time()}
        ctx.press('a', 120)
        time.sleep(1.5)
    raise RuntimeError(f'no READY TO FIGHT banner after {tries} looks: {looks}')


def ready_and_wait_searching(ctx, tag='w', cap_s=90):
    """b readies alone: its IngameCallSequenceScene reaches state 3 (PracticeScene, the waiting area), read from the
    child state-change point (x8 == 3) in b's trace. A frame of the waiting area."""
    path = _trace_path(ctx)
    start = _size(path)
    ready = _ready(ctx, tag)
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        states = _states(_trace_records(path, start))
        if any(s['kind'] == 'child' and s['x8'] == '0x3' for s in states):
            time.sleep(4.0)
            shot = ctx.shot(f'{tag}-searching')
            return _save(ctx, f'{tag}-searching-{ctx.on}', {'seconds': round(time.monotonic() - began, 1),
                                                           'ready': ready, 'shot': str(shot), 'states': states})
        time.sleep(1.0)
    shot = ctx.shot(f'{tag}-searching-timeout')
    _save(ctx, f'{tag}-searching-{ctx.on}', {'timeout': cap_s, 'states': _states(_trace_records(path, start))})
    raise RuntimeError(f'{ctx.on}: no PracticeScene (child state 3) within {cap_s} s: {shot}')


# --- the row under test -----------------------------------------------------------------------------------------------

def _tab_shown(ctx, shot):
    return _diff(ctx, shot, REFS / 'leave-battle-tab.png', TAB_BOX) < 12


def _dialog_yes(ctx, tag):
    """A centered two-button dialog is up: Yes is the right-hand button in this game's dialogs (Rematch?, Exit
    without saving?). Right, then A. Returns the frames."""
    shots = [str(ctx.shot(f'{tag}-dialog'))]
    ctx.press('right', 120)
    time.sleep(0.5)
    shots.append(str(ctx.shot(f'{tag}-dialog-right')))
    ctx.press('a', 120)
    time.sleep(1.0)
    shots.append(str(ctx.shot(f'{tag}-dialog-a')))
    return shots


def _last_waiting_entry_ns(ctx, inst):
    """Wall time (ns) of `inst`'s latest IngameCallSequenceScene change to state 3 (its PracticeScene began)."""
    last = None
    for r in _trace_records(ctx.runner.instances[inst].get('trace_jsonl') or '', 0):
        if _kind(r) == 'child' and (r.get('registers') or {}).get('x8') == '0x3':
            last = int(r.get('wall_time_ns') or 0)
    return last


def ready_and_try_leave(ctx, tag='m', expect=None, cap_s=45, settle_s=20, host_wait_s=None):
    """a readies; then, until a match session starts on a (detector) or `cap_s`: look every 0.5 s; when the game's
    waiting menu is open, Up to "Leave Battle", A, Yes; when it is not, Plus (opens it) at most once a second.
    Outcome `left` (a left the search before any session started, and none started within `settle_s`) or `matched`
    (a session started). `expect` ('left' or 'matched') fails the row on the other outcome; evidence either way.
    `host_wait_s`: Ready only once the peer's waiting area has run that long (the peer's own countdown); the peer's
    actual wait at Ready is recorded either way."""
    runner, on = ctx.runner, ctx.on
    det = runner.instances[on]['detector_log']
    det_start = _size(det)
    peer = 'b' if on == 'a' else 'a'
    peer_det = runner.instances[peer]['detector_log']
    peer_start = _size(peer_det)
    traces = {k: _size(runner.instances[k].get('trace_jsonl') or '') for k in (on, peer)}
    out = {'tag': tag, 'expect': expect, 'events': [], 'host_wait_s': host_wait_s}
    entered = _last_waiting_entry_ns(ctx, peer)
    if host_wait_s and entered:
        remaining = host_wait_s - (time.time_ns() - entered) / 1e9
        if remaining > 0:
            time.sleep(remaining)
    out['peer_waiting_s_at_ready'] = round((time.time_ns() - entered) / 1e9, 1) if entered else None
    out['ready'] = _ready(ctx, tag)
    began = time.monotonic()
    last_plus = 0.0
    left_at = None
    i = 0
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        if SESSION.search(_tail(det, det_start)):
            out['events'].append({'t': round(time.monotonic() - began, 2), 'what': 'session'})
            break
        shot = ctx.shot(f'{tag}-look-{i:03d}')
        i += 1
        t = round(time.monotonic() - began, 2)
        if _tab_shown(ctx, shot):
            out['events'].append({'t': t, 'what': 'menu', 'shot': str(shot)})
            ctx.press('up', 120)
            time.sleep(0.6)
            tab = ctx.shot(f'{tag}-tab-focus')
            ctx.press('a', 120)
            time.sleep(1.2)
            confirm = ctx.shot(f'{tag}-after-a')
            out['events'].append({'t': round(time.monotonic() - began, 2), 'what': 'leave-a',
                                  'tab': str(tab), 'after_a': str(confirm)})
            if _tab_shown(ctx, confirm):
                continue   # the A did not leave the menu: look again
            out['events'].append({'what': 'yes', 'shots': _dialog_yes(ctx, tag)})
            left_at = time.monotonic()
            break
        if time.monotonic() - last_plus >= 1.0:
            ctx.press('plus', 100)
            last_plus = time.monotonic()
            out['events'].append({'t': t, 'what': 'plus'})
        time.sleep(0.35)
    if left_at is not None:
        end = left_at + settle_s
        while time.monotonic() < end and not SESSION.search(_tail(det, det_start)):
            ctx.check_stop()
            time.sleep(1.0)
    session = SESSION.search(_tail(det, det_start))
    out['outcome'] = 'matched' if session else ('left' if left_at is not None else 'none')
    out['frames_taken'] = i
    out['after'] = str(ctx.shot(f'{tag}-after'))
    out['peer_after'] = str(runner.shot(peer, f'{tag}-peer-after'))
    out['detector'] = {on: _tail(det, det_start).splitlines()[-6:], peer: _tail(peer_det, peer_start).splitlines()[-6:]}
    out['trace'] = {}
    for k in (on, peer):
        path = runner.instances[k].get('trace_jsonl') or ''
        (Path(ctx.row_dir) / f'{tag}-trace-{k}.jsonl').write_text(_tail(path, traces[k]))
        out['trace'][k] = _states(_trace_records(path, traces[k]))[-300:]
    _save(ctx, f'{tag}-try-leave', out)
    if expect and out['outcome'] != expect:
        raise RuntimeError(f'{tag}: outcome {out["outcome"]}, expected {expect}')
    return {'outcome': out['outcome'], 'events': len(out['events'])}


def ensure_searching(ctx, tag='w'):
    """The peer after a's leave: still searching (its last IngameCallSequenceScene state is 3, PracticeScene) or back
    at the character select (then it readies again). Records which, with a frame."""
    path = _trace_path(ctx)
    states = [s for s in _states(_trace_records(path or '', 0)) if s['kind'] in ('child', 'parent')]
    last_child = next((s for s in reversed(states) if s['kind'] == 'child'), None)
    last_parent = next((s for s in reversed(states) if s['kind'] == 'parent'), None)
    shot = ctx.shot(f'{tag}-peer-look')
    searching = bool(last_child and last_child['x8'] == '0x3' and last_parent and last_parent['x8'] == '0x8')
    out = {'searching': searching, 'last_child': last_child, 'last_parent': last_parent, 'shot': str(shot)}
    if not searching:
        out['css'] = css_to_banner(ctx)
        out['again'] = ready_and_wait_searching(ctx, tag)
    return _save(ctx, f'{tag}-ensure-{ctx.on}', out)


def ready_and_wait_session(ctx, tag='m', cap_s=120):
    """Ready, no leave attempt: wait for a match session on this instance's detector (searching again works)."""
    det = ctx.runner.instances[ctx.on]['detector_log']
    start = _size(det)
    trace = _trace_path(ctx)
    tstart = _size(trace or '')
    ready = _ready(ctx, tag)
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        m = SESSION.search(_tail(det, start))
        if m:
            return _save(ctx, f'{tag}-session-{ctx.on}', {'seconds': round(time.monotonic() - began, 1), 'line': m.group(0),
                                                         'ready': ready,
                                                         'states': _states(_trace_records(trace or '', tstart))})
        time.sleep(1.0)
    raise RuntimeError(f'{ctx.on}: no match session within {cap_s} s of Ready')


# --- matches ----------------------------------------------------------------------------------------------------------

def _vary_still(runner, on, tick):
    """Varied input that does not drift (of_steps.py): half walk left, jab, half walk right, shield."""
    kind = tick % 4
    if kind == 0:
        runner.call(on, 'input.stick', {'stick': 'left', 'x': -0.5, 'y': 0, 'lease_ms': 250})
    elif kind == 2:
        runner.call(on, 'input.stick', {'stick': 'left', 'x': 0.5, 'y': 0, 'lease_ms': 250})
    else:
        runner.call(on, 'input.button', {'button': 'a' if kind == 1 else 'zl', 'state': 'tap',
                                         'duration_ms': 80, 'lease_ms': 500})


def battle(ctx, tag='m', cap_s=300, shot_every_s=20.0):
    """Both fighters act (no idle drop) until both detectors log sync_end. Returns the lines; fails on a desync."""
    runner = ctx.runner
    logs = {k: runner.instances[k]['detector_log'] for k in ('a', 'b')}
    starts = {k: max(0, _size(p) - 4096) for k, p in logs.items()}
    ended, tick = {}, 0
    began = time.monotonic()
    next_shot = began
    try:
        while time.monotonic() - began < cap_s and len(ended) < 2:
            ctx.check_stop()
            for k in logs:
                if k not in ended:
                    _vary_still(runner, k, tick + (1 if k == 'b' else 0))
            tick += 1
            if time.monotonic() >= next_shot:
                for k in logs:
                    runner.shot(k, f'{tag}-battle-{int(time.monotonic() - began):03d}s')
                next_shot += shot_every_s
            time.sleep(0.5)
            for k, p in logs.items():
                m = SYNC_END.search(_tail(p, starts[k]))
                if m:
                    ended[k] = m.group(0)
    finally:
        for k in logs:
            runner.call(k, 'input.reset')
    out = _save(ctx, f'{tag}-battle', {'sync_end': ended, 'seconds': round(time.monotonic() - began, 1)})
    if len(ended) < 2:
        raise RuntimeError(f'no sync_end on both detectors within {cap_s} s: {ended}')
    bad = {k: v for k, v in ended.items() if 'desync_seen=0' not in v}
    if bad:
        raise RuntimeError(f'desync: {bad}')
    return out


def after_try(ctx, tag='m', wait_s=30.0):
    """After a try row: when that try ended in a match, both play it out (`battle`) and decline the Rematch? (both back
    to the character select); when a left, nothing (a is at the character select, b still searching or not:
    the next row's `ensure_searching` reads it)."""
    path = Path(ctx.row_dir).parent
    found = sorted(path.glob(f'*/{tag}-try-leave.json'))
    if not found:
        raise RuntimeError(f'no {tag}-try-leave.json in {path}')
    outcome = json.loads(found[-1].read_text()).get('outcome')
    out = {'tag': tag, 'outcome': outcome}
    if outcome == 'matched':
        out['battle'] = battle(ctx, tag)
        out['decline'] = decline_to_css(ctx, wait_s)
    else:
        out['frames'] = {k: str(ctx.runner.shot(k, f'{tag}-after-leave')) for k in ('a', 'b')}
    return _save(ctx, f'{tag}-after', out)


def decline_to_css(ctx, wait_s=30.0):
    """Neither answers 'Rematch?': both countdowns run out and both return to the character select. Frames."""
    time.sleep(wait_s)
    return {k: str(ctx.runner.shot(k, 'after-decline')) for k in ('a', 'b')}


def leave_css_to_quickplay(ctx, tries=4):
    """From the character select back to the Quickplay menu (B), looking after each press."""
    for i in range(tries):
        ctx.press('b', 120)
        time.sleep(2.5)
        shot = ctx.shot(f'leave-css-{i}')
        if _diff(ctx, shot, REFS / 'quickplay-menu.png', [40, 5, 300, 45]) <= 8:
            return {'presses': i + 1}
    raise RuntimeError(f'{ctx.on}: not on the Quickplay menu after {tries} B presses')


def z_instance(ctx):
    """End row per instance: trace.unmapped_access total 0 and no panic/abort/crash/Unmapped in the console."""
    got = ctx.call('trace.unmapped_access', {})
    text = Path(ctx.instance['console_log']).read_text(errors='replace') if ctx.instance.get('console_log') else ''
    bad = {k: len(re.findall(k, text)) for k in ('panic', 'abort', 'crash', 'Unmapped')}
    events = Path(ctx.instance['qcf_log']).read_text(errors='replace') if ctx.instance.get('qcf_log') and \
        os.path.exists(ctx.instance['qcf_log']) else ''
    out = _save(ctx, f'z-{ctx.on}', {'unmapped': got, 'console': bad, 'qcf_events': events.splitlines()[-20:]})
    if int((got or {}).get('total', -1)) != 0 or any(bad.values()):
        raise RuntimeError(f'{ctx.on}: {out}')
    return out
