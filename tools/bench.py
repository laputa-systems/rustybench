#!/usr/bin/env python3
import argparse
import datetime
import os
import pathlib
import platform
import shlex
import shutil
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
TIERS = ('Edit', 'Change', 'Scenario', 'Full')
WORKLOADS = {
    'accounting': ('Edit', 'accounting', 'allocation median contract', ['cargo', 'test', '--locked', '--lib', 'alloc_median_follows_allocation_order_not_time_order']),
    'runner': ('Change', 'runner', 'representative runner and collector, one sample', ['cargo', 'run', '--locked', '--bin', 'rustybench', '--', 'baseline', '--fast', '--baseline', '{out}/baseline.json', '--', 'cargo', 'bench', '--locked', '--bench', 'bench', '--', 'workloads::sort_records::256']),
    'overhead': ('Scenario', 'runner', 'ignored harness controls, three rounds', ['cargo', 'run', '--locked', '--bin', 'rustybench', '--', 'baseline', '--sample-count', '10', '--sample-size', '1', '--baseline', '{out}/baseline.json', '--', 'cargo', 'bench', '--locked', '--bench', 'bench', '--', '--ignored', 'harness_overhead']),
    'suite': ('Full', 'runner', 'all representative workloads, three rounds', ['cargo', 'run', '--locked', '--bin', 'rustybench', '--', 'baseline', '--sample-count', '10', '--sample-size', '1', '--baseline', '{out}/baseline.json', '--', 'cargo', 'bench', '--locked', '--bench', 'bench']),
}


def select(args):
    for kind, values, known in [('name', args.name, WORKLOADS), ('area', args.area, {w[1] for w in WORKLOADS.values()}), ('skip', args.skip, WORKLOADS)]:
        unknown = set(values) - set(known)
        if unknown:
            raise ValueError('unknown {}: {}'.format(kind, ', '.join(sorted(unknown))))
    selected, excluded = [], []
    for name, (tier, area, _, _) in WORKLOADS.items():
        reasons = []
        if TIERS.index(tier) > TIERS.index(args.tier): reasons.append('above tier ceiling')
        if args.name and name not in args.name: reasons.append('name filter')
        if args.area and area not in args.area: reasons.append('area filter')
        if name in args.skip: reasons.append('skipped')
        (excluded if reasons else selected).append((name, ', '.join(reasons)))
    if not selected:
        raise ValueError('selection is empty')
    return selected, excluded


def preflight(selected):
    names = ', '.join(name for name, _ in selected)
    errors = []
    for tool in ('cargo', 'rustc', 'git'):
        if not shutil.which(tool): errors.append('{} missing; install Git and the pinned Rust toolchain (needed by {})'.format(tool, names))
    if platform.system() not in ('Darwin', 'Linux'):
        errors.append('unsupported platform; use macOS or Linux (needed by {})'.format(names))
    if shutil.disk_usage(ROOT).free < 1024 ** 3:
        errors.append('less than 1 GiB free; free disk space (needed by {})'.format(names))
    paths = ['Cargo.lock', 'rust-toolchain.toml']
    if any(WORKLOADS[name][1] == 'runner' for name, _ in selected): paths.append('benches/bench.rs')
    for path in paths:
        if not (ROOT / path).is_file(): errors.append('{} missing; restore the checkout (needed by {})'.format(path, names))
    if errors:
        raise ValueError('\n'.join(errors))
    for tool in ('cargo', 'rustc', 'git'):
        result = subprocess.run([tool, '--version'], cwd=ROOT, capture_output=True, text=True)
        if result.returncode: raise ValueError('{} unavailable; repair Git or the pinned Rust toolchain (needed by {}): {}'.format(tool, names, result.stderr.strip()))
        print(result.stdout.strip())
    print('host: {} {}; free disk: {} GiB; isolation and load not verified'.format(platform.system(), platform.machine(), shutil.disk_usage(ROOT).free // 1024 ** 3))
    print('Cargo rebuilds stale artifacts; no external fixtures or services required.')


def main():
    parser = argparse.ArgumentParser(description='Tiered Rustybench workloads; Full requires explicit selection.')
    parser.add_argument('--tier', choices=TIERS, default='Edit')
    parser.add_argument('--name', action='append', default=[])
    parser.add_argument('--area', action='append', default=[])
    parser.add_argument('--skip', action='append', default=[])
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--list', action='store_true')
    args = parser.parse_args()
    if args.list:
        for name, (tier, area, purpose, command) in WORKLOADS.items():
            print('{} | {} | {} | {} | {}'.format(name, tier, area, purpose, shlex.join(command)))
        return 0
    try:
        selected, excluded = select(args)
        for name, reason in excluded: print('excluded {}: {}'.format(name, reason))
        if args.dry_run:
            for name, _ in selected: print('{}: {}'.format(name, shlex.join(WORKLOADS[name][3])))
            return 0
        preflight(selected)
        if args.check: return 0
        run_dir = ROOT / '.work' / 'bench' / datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f')
        run_dir.mkdir(parents=True)
        summary = ['commit: ' + subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'dirty: ' + ('yes' if subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip() else 'no'), 'host: {} {}'.format(platform.system(), platform.machine()), 'timing validity: unqualified; host load and discarded samples not assessed', 'no budgets: completion is not a performance verdict']
        summary += ['excluded {}: {}'.format(name, reason) for name, reason in excluded]
        failed = False
        for name, _ in selected:
            out = run_dir / name
            out.mkdir()
            command = [part.replace('{out}', str(out)) for part in WORKLOADS[name][3]]
            start = time.monotonic()
            with (out / 'run.log').open('w') as log:
                result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
            duration = time.monotonic() - start
            line = '{}: exit {} duration {:.2f}s; {}'.format(name, result.returncode, duration, shlex.join(command))
            print(line, flush=True)
            summary.append(line)
            failed |= result.returncode != 0
        (run_dir / 'summary.txt').write_text('\n'.join(summary) + '\n')
        print('summary: {}'.format(run_dir / 'summary.txt'))
        return int(failed)
    except (ValueError, OSError) as error:
        print('bench: {}'.format(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
