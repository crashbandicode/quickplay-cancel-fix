"""Host tests for tools/verdict.py, tools/gifcheck.py and tools/timeline.py (python3 -m unittest)."""
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
