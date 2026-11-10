#!/bin/bash
# Remote (XPS): run the freeze A/B arms back to back (primary 17:59Z/18:13Z). First waits for the running arm's watcher
# to end (its freeze-watch.log); then for each <label>:<arm>: Down the previous run, stage, Up, start the runner and
# watch it (freeze_watch.sh). Stops at the first stall, leaving that run Up for inspection (the caller Downs it), or on
# any staging/Up failure. Writes one line per event to /home/intpa/eden-runs/fz-chain.log.
# Usage: fz_chain.sh <running label> <label>:<V|P|R|N> ...
set -u
export PATH=/usr/bin:$PATH
in=/home/intpa/eden-runs/inputs-qcf
log=/home/intpa/eden-runs/fz-chain.log
say() { echo "$(date -u +%H:%M:%S)Z $*" >> "$log"; }
prev="$1"; shift
say "chain start: wait for $prev, then $*"
while ! grep -qE 'runner finished|STALL|time cap' "/home/intpa/eden-runs/$prev/freeze-watch.log" 2>/dev/null; do sleep 5; done
sleep 20
if grep -q STALL "/home/intpa/eden-runs/$prev/freeze-watch.log"; then
  say "STOP: $prev stalled ($(grep STALL "/home/intpa/eden-runs/$prev/freeze-watch.log" | head -n 1)); left Up"
  exit 3
fi
say "$prev finished: $(grep 'runner finished' "/home/intpa/eden-runs/$prev/freeze-watch.log" | cut -c1-140)"
for item in "$@"; do
  label="${item%%:*}"; arm="${item##*:}"
  bash "$in/down.sh" "$prev" > "/home/intpa/eden-runs/$prev/chain-down.log" 2>&1
  say "$prev down: $(grep -E 'down rc|released' "/home/intpa/eden-runs/$prev/chain-down.log" | tr '\n' ' ' | cut -c1-160)"
  if pgrep -x eden >/dev/null; then say "STOP: Eden still running after $prev's Down"; exit 4; fi
  bash "$in/stage-fz.sh" "$label" "$arm" > "/tmp/stage-$label.log" 2>&1
  if ! grep -q 'preflight rc=0' "/tmp/stage-$label.log"; then say "STOP: $label staging failed: $(tail -n 3 /tmp/stage-$label.log | tr '\n' ' ')"; exit 5; fi
  say "$label ($arm) staged: $(grep -E '^a points|^b points' "/tmp/stage-$label.log" | tr '\n' ' ' | cut -c1-200)"
  bash "$in/up.sh" "$label" > "/tmp/up-$label.log" 2>&1
  if ! grep -q 'up rc=0' "/tmp/up-$label.log"; then say "STOP: $label Up failed: $(head -n 3 /tmp/up-$label.log | tr '\n' ' ')"; exit 6; fi
  say "$label ($arm) Up: $(grep 'up rc' "/tmp/up-$label.log")"
  bash "$in/freeze_watch.sh" "$label" 30 > "/home/intpa/eden-runs/$label/freeze-watch.log" 2>&1
  rc=$?
  if [ $rc -eq 3 ]; then
    say "STOP: $label ($arm) STALL captured: $(grep -E 'STALL|captured' "/home/intpa/eden-runs/$label/freeze-watch.log" | tr '\n' ' ' | cut -c1-240); left Up"
    exit 3
  fi
  say "$label ($arm) watcher rc=$rc: $(tail -n 1 "/home/intpa/eden-runs/$label/freeze-watch.log" | cut -c1-160)"
  sleep 15
  prev="$label"
done
bash "$in/down.sh" "$prev" > "/home/intpa/eden-runs/$prev/chain-down.log" 2>&1
say "$prev down: $(grep -E 'down rc|released' "/home/intpa/eden-runs/$prev/chain-down.log" | tr '\n' ' ' | cut -c1-160)"
say "chain done"
