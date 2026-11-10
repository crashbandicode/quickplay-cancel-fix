"""Host tests for tools/verdict.py, tools/gifcheck.py and tools/timeline.py (python3 -m unittest)."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import timeline  # noqa: E402
import verdict  # noqa: E402


def rec(location, frame, registers=None, captures=None):
    return {'locations': {'pc': location}, 'frame': frame, 'registers': registers or {},
            'captures': captures or []}


def state(frame, old, new):
    return rec('main+0x18514bc', frame, {'x8': hex(new), 'x9': hex(old)},
               [{'type': 'u32', 'value': 0}, {'type': 'u32', 'value': 0}])


def leave_state(frame, mask):
    return rec('main+0x18518b0', frame, captures=[{'type': 'u32', 'value': 1}, {'type': 'u32', 'value': mask}])


def host_start(frame, counter):
    return rec('main+0x1566af4', frame, {'x1': '0x1'}, [{'type': 'u32', 'value': counter}])


class Verdicts(unittest.TestCase):
    def test_kinds_come_from_locations(self):
        self.assertEqual(timeline.kind_of(host_start(1, 0)), 'host_round_start')
        self.assertEqual(timeline.kind_of(rec('main+0x1566758', 1)), 'host_branch')
        self.assertIsNone(timeline.kind_of(rec('main+0x1', 1)))

    def test_round_start_paths_split_on_the_full_room_counter(self):
        trace = [host_start(100, 3), host_start(400, 180), rec('main+0x1566af4', 500)]
        self.assertEqual(verdict.round_start_paths(trace),
                         [(100, 'countdown', 3), (400, 'full_room', 180), (500, 'unknown', None)])

    def test_leave_outcomes_reads_accept_and_refuse(self):
        trace = [state(10, 0, 1),
                 state(20, 1, 4), leave_state(21, 1), state(21, 4, 1),     # refused: a round had started
                 state(50, 1, 4), leave_state(51, 0), state(52, 4, 2),     # accepted
                 state(90, 1, 4)]                                          # still open at the end
        self.assertEqual(verdict.leave_outcomes(trace),
                         [(20, 'refused', 1), (50, 'accepted', 0), (90, 'open', None)])

    def test_desync_clean_needs_a_clean_sync_end(self):
        clean = 'session slot=0 kind=online\nsync_end trigger=result desync_seen=0\n'
        self.assertEqual(verdict.desync_clean(clean), (True, 1, 1))
        self.assertEqual(verdict.desync_clean(clean + 'sync_end trigger=x desync_seen=2\n'), (False, 1, 2))
        self.assertEqual(verdict.desync_clean('session slot=0 kind=online\n'), (False, 1, 0))

    def test_online_sessions_split_clean_cut_and_desync(self):
        old_boot = ('1 plugin=ssbu-desync-detector/0.1.0 hooks=lockstep\n'
                    '2 session slot=1 kind=online heap_allocations=0\n'
                    '3 frame_gap kind=online slot=1 frame=10 last_frame_tick=2\n')
        text = old_boot + (
            '9 plugin=ssbu-desync-detector/0.2.0 telemetry=1 heap_allocations=0\n'
            '10 session slot=4 kind=playback heap_allocations=0\n'
            '11 session slot=5 kind=online heap_allocations=0\n'
            '12 sync_end trigger=other kind=0 frame=7542 local=0 desync_seen=0 already_ending=0\n'
            '13 session slot=6 kind=online heap_allocations=0\n'
            '14 frame_gap kind=online slot=6 frame=2108 last_frame_tick=13\n'
            '15 sync_end trigger=normal kind=1 frame=2111 local=0 desync_seen=0 already_ending=0\n'
            '16 session slot=7 kind=online heap_allocations=0\n'
            '17 telemetry boot_tick=9 result=0x0\n'
            '18 frame_gap kind=online slot=7 frame=2020 last_frame_tick=16\n'
            '19 session slot=0 kind=online heap_allocations=0\n'
            '20 sync_end trigger=normal kind=1 frame=900 local=0 desync_seen=3 already_ending=0\n'
            '21 session slot=1 kind=online heap_allocations=0\n')
        self.assertEqual(verdict.online_sessions(text), [
            (5, 'clean', 'other/0@7542', None),
            (6, 'clean', 'normal/1@2111', 2108),
            (7, 'cut', None, 2020),
            (0, 'desync', 'normal/1@900', None),
            (1, 'open', None, None)])
        # Only the last boot counts; without an install line every session does.
        self.assertEqual(verdict.online_sessions(old_boot), [(1, 'cut', None, 10)])
        self.assertEqual(verdict.online_sessions(old_boot.split('\n', 1)[1]), [(1, 'cut', None, 10)])

    def test_plugin_log_reads_installs_and_adopted_counts(self):
        text = ('6265036295921 plugin=quickplay-cancel-fix/0.2.0 build=diag wait=stock/near result=installed '
                'heap_allocations=0\n'
                '6269490145603 diag adopted slot=1 count=1->2\n'
                '6283650953987 diag adopted slot=0 count=1->2\n'
                '7 plugin=quickplay-cancel-fix/0.2.0 wait=stock/near result=installed heap_allocations=0\n'
                '8 plugin=quickplay-cancel-fix/0.2.0 site=count_bump result=not_installed heap_allocations=0\n')
        installs, adopted = verdict.plugin_log(text)
        self.assertEqual(installs, [('0.2.0', 'diag', 'installed'), ('0.2.0', 'release', 'installed'),
                                    ('0.2.0', 'release', 'not_installed')])
        self.assertEqual(adopted, [(1, 1, 2), (0, 1, 2)])

    def test_leave_try_needs_the_tab_with_the_menu_open(self):
        left = {'tag': 'af', 'outcome': 'left', 'looks': [
            {'t': 0.0, 'menu': False, 'tab': False},
            {'t': 1.29, 'menu': True, 'tab': True},
            {'t': 5.4, 'tab': True, 'left': True, 'what': 'leave'}]}
        self.assertEqual(verdict.leave_try(left), ('af', True, 1.29, 'left'))
        hidden = {'tag': 'bf', 'outcome': 'matched', 'looks': [
            {'t': 0.0, 'menu': False, 'tab': False},
            {'t': 1.28, 'menu': True, 'tab': False},
            {'t': 21.63, 'menu': None, 'tab': None}]}
        self.assertEqual(verdict.leave_try(hidden), ('bf', False, None, 'matched'))
        # A leave record alone (no look with the menu open) is not a sighting.
        self.assertEqual(verdict.leave_try({'tag': 'x', 'outcome': 'none', 'looks': [
            {'t': 3.0, 'tab': True, 'what': 'leave'}]}), ('x', False, None, 'none'))
        with tempfile.TemporaryDirectory() as tmp:
            for name, data in (('after/af-try-leave.json', left), ('before/bf-try-leave.json', hidden)):
                path = Path(tmp) / name
                path.parent.mkdir()
                path.write_text(json.dumps(data))
            self.assertEqual(verdict.leave_tries(tmp), [('af', True, 1.29, 'left'), ('bf', False, None, 'matched')])

    def test_check_gif_measures_width_and_length(self):
        try:
            from PIL import Image
        except ImportError:
            self.skipTest('Pillow missing')
        import gifcheck
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 't.gif'
            frames = [Image.new('RGB', (1280, 720), (i * 40, 0, 0)) for i in range(3)]
            frames[0].save(path, save_all=True, append_images=frames[1:], duration=[1000, 2000, 500], loop=0)
            ok, width, seconds, count = gifcheck.check_gif(path)
            self.assertEqual((ok, width, count), (True, 1280, 3))
            self.assertAlmostEqual(seconds, 3.5)
            frames[0].save(path, save_all=True, append_images=frames[1:], duration=[5000, 5000, 100], loop=0)
            self.assertFalse(gifcheck.check_gif(path)[0])


if __name__ == '__main__':
    unittest.main()
