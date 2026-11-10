#!/bin/bash
# Remote (XPS): stage one freeze A/B arm (primary 17:35Z/17:59Z/18:13Z; never Up): the pair on the XPS (a :124, b :125),
# the research Eden version (xps-base-rmc.json), profile y's qt-config with LAN TLS on for both instances, the CPU pilot
# (auto rematch) on both; arm V no QCF, arm P QCF v0.2.0 release, arms R/N the candidate NROs in inputs-qcf/cand-<r|n>.
# QCF arms use rows-fz02 (b with the GDB stub, untraced) and trace a's count compare .1563f04 too.
# Usage: stage-fz.sh <label> <V|P|R|N>
set -u
export PATH=/usr/bin:$PATH
in=/home/intpa/eden-runs/inputs-qcf
label="$1"; arm="$2"
cd "$in" || exit 1
extra=(); rows=rows-fz01; what='no QCF'
case "$arm" in
  V) ;;
  P) extra=(--nro "a=$in/qcf020r/libquickplay_cancel_fix.nro"); what='QCF v0.2.0 release 0b12208d';;
  R) extra=(--nro "a=$in/cand-r/libquickplay_cancel_fix.nro"); what="QCF candidate R $(sha256sum cand-r/libquickplay_cancel_fix.nro | cut -c1-8) (research, not released)";;
  N) extra=(--nro "a=$in/cand-n/libquickplay_cancel_fix.nro"); what="QCF candidate N $(sha256sum cand-n/libquickplay_cancel_fix.nro | cut -c1-8) (research, not released)";;
  *) echo "arm $arm?"; exit 1;;
esac
if [ "$arm" != V ]; then
  rows=rows-fz02; extra+=(--untraced b --gate-point)
fi
mkdir -p "$rows/refs"
cp -p rows-obs01/refs/* "$rows/refs/" 2>/dev/null
cp -p waiting-menu.png "$rows/refs/" 2>/dev/null
auth="QCF freeze A/B arm $arm (primary 17:35Z/17:59Z/18:13Z: an opponent freeze seen in qcf-srm12 and qcf-sfz1 at the first native rematch match after a v0.2.0 first match; research only, nothing released): the research Eden version 20261110-656ca2fcd596-unmapped-stop-lan2c-rmcdump-52288a74e874 (test only, never current); both instances detector + tile cursor + ssbu-cpu-pilot 0.2.2 release ed8f308f (auto_rematch=on, P1 the player, no piloting); a also $what; rows m1 (first match from the CSS) and ry1 (yes/yes rematch match played to its end); $([ "$arm" != V ] && echo 'b with the Eden GDB stub (untraced); ')Quickplay against each other only (LAN Online, TLS on both). Volume 0, no Switch SD."
bash restage.sh "$label" "$rows" "config-$label-src.json" \
  --overlay "$in/overlay/qt-config-y-tls.ini@user/config/qt-config.ini" --host-base "$in/xps-base-rmc.json" \
  --overlay "$in/pilot/config-auto-rematch.ini@user/sdmc/ultimate/ssbu-cpu-pilot/config.ini" \
  --nro "a=$in/pilot/libssbu_cpu_pilot.nro" --nro "b=$in/pilot/libssbu_cpu_pilot.nro" "${extra[@]}" \
  --authorization "$auth" 2>&1 | tail -n 6
cfg="/home/intpa/eden-runs/$label/config.json"
python3 - "$cfg" <<'EOF'
import json, sys
c = json.load(open(sys.argv[1]))
for i in c['instances']:
    pts = (i.get('trace') or {}).get('points') or []
    print(i['key'], 'points', len(pts), 'gate' if any(p['target'] == 'main+0x1563f04' for p in pts) else '-',
          [p['path'].rsplit('/', 1)[1] for p in i['plugins']], i['app']['path'].split('/')[-2][-12:])
EOF
grep -h 'lan_online_tls=' "$in/overlay/qt-config-y-tls.ini"
