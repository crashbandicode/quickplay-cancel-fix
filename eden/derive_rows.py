"""Derive an "after" rows folder from the observe/before one: the same rows and hooks, plus a gate that the QCF
NRO installed on the instances that carry it (its install line in `qcf_log`, heap_allocations=0).

Usage: derive_rows.py <src rows dir> <dst rows dir> --on b [--on a] --about "..." --scope "..."
Copies rows.json (edited), steps.py and segments/; refs/ (game-derived, not in git) are copied when present.
"""
import argparse
import json
import shutil
from pathlib import Path

INSTALLED = r'plugin=quickplay-cancel-fix/\S+ [^\n]*result=installed heap_allocations=0'


def derive(rows, on, about, scope):
    rows = json.loads(json.dumps(rows))
    rows['about'] = about
    rows['scope'] = scope
    rows['gates']['qcf-installed'] = {'kind': 'log', 'source': 'qcf_log', 'whole_file': True, 'cap': 120,
                                      'pattern': INSTALLED}
    load = next(r for r in rows['rows'] if r['id'] == 'q00-load')
    at = max(i for i, s in enumerate(load['steps']) if s.get('gate') == 'installed') + 1
    load['steps'][at:at] = [{'gate': 'qcf-installed', 'on': key} for key in on]
    rows['setup'][0]['ref'] = ("pair config: instances a and b (rc6-bg-qp6's): detector + tile cursor, the QCF NRO on "
                               + ' and '.join(on))
    return rows


def main():
    p = argparse.ArgumentParser()
    p.add_argument('src')
    p.add_argument('dst')
    p.add_argument('--on', action='append', required=True)
    p.add_argument('--about', required=True)
    p.add_argument('--scope', required=True)
    args = p.parse_args()
    src, dst = Path(args.src), Path(args.dst)
    dst.mkdir(parents=True, exist_ok=True)
    rows = derive(json.loads((src / 'rows.json').read_text()), args.on, args.about, args.scope)
    (dst / 'rows.json').write_text(json.dumps(rows, indent=2) + '\n')
    shutil.copy2(src / 'steps.py', dst / 'steps.py')
    for sub in ('segments', 'refs'):
        if (src / sub).is_dir():
            shutil.copytree(src / sub, dst / sub, dirs_exist_ok=True)
    print(json.dumps({'dst': str(dst), 'rows': len(rows['rows']), 'qcf_on': args.on}))


if __name__ == '__main__':
    main()
