#!/bin/bash
# Remote (XPS): the freeze A/B's watcher (as surface/freeze_watch.ps1). While a run is Up, watch each instance's
# desync-detector events.log (a plugin thread logs telemetry every 5 s). Once it was written after the Up but not for
# STALE seconds while that instance's Eden still runs, capture its evidence before anything stops, into
# $run/freeze-<k>-<HHMMSS>/: every host thread's state, CPU ticks and wchan twice 10 s apart, the control socket's
# trace.counters twice, screenshots of both instances, the tails of its trace, console and detector logs; then, if b
# runs Eden's GDB stub (out/gdb/hold.log), guest PC/LR/backtrace samples through gdb_sample.py. Never stops a process.
# Exit 0: the runner finished with no stall; 3: a stall was captured; 1: the time cap.
# Usage: freeze_watch.sh <label> [minutes=30] [stale seconds=30]
set -u
export PATH=/usr/bin:$PATH
label="$1"; minutes="${2:-30}"; stale="${3:-30}"
run="/home/intpa/eden-runs/$label"
ctl=/home/intpa/Applications/Eden-GIF/20261110-656ca2fcd596-unmapped-stop-lan2c-rmcdump-52288a74e874/tools/edenctl.py
up_at=$(stat -c %Y "$run/state/instances.json")
end=$(( $(date +%s) + minutes * 60 ))
edenpid() { pgrep -f -x ".*/usr/bin/eden .*$label/state/run-$1/.*" | head -n 1; }
threads() { for t in /proc/$1/task/*; do
  read -r st < "$t/stat"; rest="${st##*) }"; set -- $rest
  w=$(cat "$t/wchan" 2>/dev/null); c=$(tr " " _ < "$t/comm")
  echo "$(basename "$t") $c $1 $(( ${12} + ${13} )) ${w:--}"; done; }
capture() {
  local k="$1" peer dir pid
  [ "$k" = a ] && peer=b || peer=a
  dir="$run/freeze-$k-$(date -u +%H%M%S)"; mkdir -p "$dir"
  pid=$(edenpid "$k")
  echo "freeze capture for $k (pid $pid) started $(date -u +%H:%M:%S)Z" > "$dir/README.txt"
  timeout 30 python3 -I "$ctl" --socket "$run/state/run-$k/control.sock" call trace.counters '{}' > "$dir/trace-counters-1.json" 2>&1
  threads "$pid" > "$dir/threads-1.txt"
  timeout 30 python3 -I "$ctl" --socket "$run/state/run-$k/control.sock" call ui.screenshot "{\"target\":\"all\",\"path\":\"$dir/$k-frame-1.png\"}" > "$dir/shot-1.json" 2>&1
  timeout 30 python3 -I "$ctl" --socket "$run/state/run-$peer/control.sock" call ui.screenshot "{\"target\":\"all\",\"path\":\"$dir/$peer-frame.png\"}" > "$dir/peer-shot.json" 2>&1
  sleep 10
  threads "$pid" > "$dir/threads-2.txt"
  timeout 30 python3 -I "$ctl" --socket "$run/state/run-$k/control.sock" call trace.counters '{}' > "$dir/trace-counters-2.json" 2>&1
  timeout 30 python3 -I "$ctl" --socket "$run/state/run-$k/control.sock" call ui.screenshot "{\"target\":\"all\",\"path\":\"$dir/$k-frame-2.png\"}" > "$dir/shot-2.json" 2>&1
  timeout 30 python3 -I "$ctl" --socket "$run/state/run-$k/control.sock" call status '{}' > "$dir/status.json" 2>&1
  join -j1 <(sort "$dir/threads-1.txt") <(sort "$dir/threads-2.txt") | awk '{printf "%s %s %s +%d/10s wchan=%s\n", $1, $2, $7, $8-$4, $9}' \
    | sort -t+ -k2 -nr > "$dir/threads.txt"
  tail -n 40 "$run/state/run-$k/$k.trace.jsonl" > "$dir/$k-trace-tail.jsonl" 2>/dev/null
  tail -n 60 "$run/state/run-$k/console.log" > "$dir/$k-console-tail.log" 2>/dev/null
  for x in a b; do
    tail -n 30 "$run/work/profiles/profile-$x/user/sdmc/ultimate/ssbu-desync-detector/events.log" > "$dir/$x-events-tail.log" 2>/dev/null
  done
  echo "freeze capture for $k done $(date -u +%H:%M:%S)Z" >> "$dir/README.txt"
  echo "$dir"
}
while [ "$(date +%s)" -lt "$end" ]; do
  if grep -q '^{"status": ' "$run/run.out" 2>/dev/null; then
    echo "$(date -u +%H:%M:%S)Z runner finished: $(grep '^{"status": ' "$run/run.out" | tail -n 1 | cut -c1-120)"
    exit 0
  fi
  for k in a b; do
    log="$run/work/profiles/profile-$k/user/sdmc/ultimate/ssbu-desync-detector/events.log"
    [ -f "$log" ] || continue
    last=$(stat -c %Y "$log"); now=$(date +%s)
    [ "$last" -gt "$up_at" ] || continue
    if [ $(( now - last )) -gt "$stale" ] && [ -n "$(edenpid "$k")" ]; then
      echo "$(date -u +%H:%M:%S)Z STALL: $k's detector log last written $(date -u -d @"$last" +%H:%M:%S)Z ($(( now - last )) s ago); capturing"
      dir=$(capture "$k")
      echo "captured: $dir"
      head -n 6 "$dir/threads.txt"
      if [ "$k" = b ] && [ -f "$run/out/gdb/hold.log" ]; then
        timeout 600 python3 -I /home/intpa/eden-runs/inputs-qcf/gdb_sample.py "$run" "$dir" > "$dir/gdb-sample.txt" 2>&1
        echo "gdb sample rc=$?"; tail -n 25 "$dir/gdb-sample.txt"
      fi
      exit 3
    fi
  done
  sleep 5
done
echo "$(date -u +%H:%M:%S)Z time cap"
exit 1
