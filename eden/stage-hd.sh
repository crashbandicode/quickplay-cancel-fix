#!/bin/bash
# Remote (XPS): stage the QCF 0.3.0 hold boot (primary 18:17Z/20:48Z; never Up): the pair on the XPS (a :124, b :125),
# the research Eden version (xps-base-rmc.json), LAN TLS on for both, the CPU pilot (auto rematch) on both, QCF 0.3.0
# (inputs-qcf/v030) on a with config.ini hold_seconds=3 (the rows change it per match), both instances traced (b is the
# vanilla host whose countdown and round start the rows read) with the menu gate verdict point. Rows rows-hd01.
# Usage: stage-hd.sh <label> [nro folder under inputs-qcf, default v030]
set -u
export PATH=/usr/bin:$PATH
in=/home/intpa/eden-runs/inputs-qcf
label="$1"
cd "$in" || exit 1
nro="$in/${2:-v030}/libquickplay_cancel_fix.nro"
sha=$(sha256sum "$nro" | cut -c1-8)
auth="QCF 0.3.0 hold boot (primary 18:17Z/20:48Z: the joiner holds its ready report for hold_seconds, then reports 7 through the game's setter; research, nothing released): the research Eden version 20261110-656ca2fcd596-unmapped-stop-lan2c-rmcdump-52288a74e874 (test only, never current); both instances detector + tile cursor + ssbu-cpu-pilot 0.2.2 release ed8f308f (auto_rematch=on, P1 the player, no piloting); a also quickplay-cancel-fix 0.3.0 $sha (not released) with config.ini hold_seconds 3/6/10/0 written by the rows; both traced; rows: a joins b's room from the CSS and leaves during the hold, held matches at 3, 6, 10 s (fresh and past countdowns) played out, a rematch after a held match played out, leaves during rematch holds, the hold off as the before. Quickplay against each other only (LAN Online, TLS on both). Volume 0, no Switch SD."
bash restage.sh "$label" rows-hd01 "config-$label-src.json" \
  --overlay "$in/overlay/qt-config-y-tls.ini@user/config/qt-config.ini" --host-base "$in/xps-base-rmc.json" \
  --overlay "$in/pilot/config-auto-rematch.ini@user/sdmc/ultimate/ssbu-cpu-pilot/config.ini" \
  --overlay "$in/overlay/qcf-hold-3.ini@user/sdmc/ultimate/quickplay-cancel-fix/config.ini" \
  --nro "a=$in/pilot/libssbu_cpu_pilot.nro" --nro "b=$in/pilot/libssbu_cpu_pilot.nro" --nro "a=$nro" --gate-point \
  --authorization "$auth" 2>&1 | tail -n 6
cfg="/home/intpa/eden-runs/$label/config.json"
python3 - "$cfg" <<'EOF'
import json, sys
c = json.load(open(sys.argv[1]))
for i in c['instances']:
    pts = (i.get('trace') or {}).get('points') or []
    print(i['key'], 'points', len(pts), [p['path'].rsplit('/', 1)[1] for p in i['plugins']],
          [f['to'].rsplit('/', 2)[-2] for f in i.get('files') or []], i['app']['path'].split('/')[-2][-12:])
EOF
grep -h 'lan_online_tls=' "$in/overlay/qt-config-y-tls.ini"
