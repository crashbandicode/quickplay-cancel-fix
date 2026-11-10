#!/bin/bash
# Remote (XPS): one line of a freeze A/B run's state: rows done (receipts), each instance's detector log age, the
# watcher's log tail, Eden count. Usage: fzstatus.sh <label>
run="/home/intpa/eden-runs/$1"
now=$(date +%s)
rows=$(cd "$run/out" 2>/dev/null && for f in *.json; do python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(d.get('id','?')+':'+str(d.get('status','?')))" "$f" 2>/dev/null; done | grep -v '?' | tr '\n' ' ')
ages=""
for k in a b; do
  log="$run/work/profiles/profile-$k/user/sdmc/ultimate/ssbu-desync-detector/events.log"
  [ -f "$log" ] && ages="$ages $k=$(( now - $(stat -c %Y "$log") ))s"
done
echo "$(date -u +%H:%M:%S)Z eden=$(pgrep -c -x eden) det_age:$ages rows: $rows"
tail -n 4 "$run/freeze-watch.log" 2>/dev/null
grep -h '^{"status": ' "$run/run.out" 2>/dev/null | tail -n 1 | cut -c1-160
