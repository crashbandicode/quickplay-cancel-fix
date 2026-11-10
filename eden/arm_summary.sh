#!/bin/bash
# Remote (XPS): a freeze A/B arm's evidence summary: each instance's member state reports (caller, time), the rematch
# try's outcome, a's count compare (.1563f04) runs, a's per-slot counts around the rematch, and any freeze capture.
# Usage: arm_summary.sh <label>
set -u
cd "/home/intpa/eden-runs/$1" || exit 1
in=/home/intpa/eden-runs/inputs-qcf
for k in a b; do
  f="state/run-$k/$k.trace.jsonl"; [ -f "$f" ] || f="out/logs/$k-$k.trace.jsonl"
  echo "== $k member reports ($f)"
  [ -f "$f" ] && python3 -I "$in/member_reports.py" "$f" || echo "(no trace: untraced)"
done
echo "== ry1 try: $(head -c 160 out/ry1/ry1-try-leave.json 2>/dev/null)"
f="state/run-a/a.trace.jsonl"; [ -f "$f" ] || f="out/logs/a-a.trace.jsonl"
echo "== a count compare .1563f04"
python3 -I "$in/gate_view.py" "$f" | tail -n 25
echo "== a slots (cnt1 = a's own slot when a joins)"
python3 -I "$in/slots_view.py" "$f" 2>/dev/null | cut -c1-80 | awk '{print $1, $4, $5, $6, $7}' | uniq -f1 | tail -n 12
ls -d freeze-* 2>/dev/null
for k in a b; do
  e="out/logs/$k-ssbu-desync-detector-events.log"; [ -f "$e" ] || e="work/profiles/profile-$k/user/sdmc/ultimate/ssbu-desync-detector/events.log"
  echo "== $k detector: $(grep -vc telemetry "$e" 2>/dev/null) lines; $(grep -E 'sync_end|desync|frame_gap|communication' "$e" 2>/dev/null | tail -n 4 | cut -c1-110 | tr '\n' '|')"
done
