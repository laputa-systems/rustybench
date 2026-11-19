import os
import pathlib
import shutil
import tempfile
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
DRIVER = ROOT / 'tools' / 'bench.py'


def run(*args, env=None):
    return subprocess.run([sys.executable, str(DRIVER), *args], cwd=ROOT, env=env, text=True, capture_output=True)


class BenchSelection(unittest.TestCase):
    def test_selection_and_exclusions(self):
        result = run('--tier', 'Change', '--area', 'runner', '--dry-run')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('runner: cargo', result.stdout)
        self.assertIn('excluded accounting: area filter', result.stdout)
        self.assertIn('excluded overhead: above tier ceiling', result.stdout)
        self.assertNotIn('suite: cargo', result.stdout)
        self.assertEqual(run('--tier', 'Change', '--name', 'runner', '--skip', 'runner', '--dry-run').returncode, 1)
        self.assertEqual(run('--name', 'runner', '--dry-run').returncode, 1)
        self.assertEqual(run('--name', 'unknown', '--dry-run').returncode, 1)
        self.assertEqual(run('--area', 'runner', '--name', 'accounting', '--dry-run').returncode, 1)
        self.assertEqual(run('--area', 'unknown', '--dry-run').returncode, 1)
        self.assertEqual(run('--skip', 'unknown', '--dry-run').returncode, 1)

    def test_dry_run_and_failed_preflight_do_not_create_runs(self):
        directory = ROOT / '.work' / 'bench'
        before = set(directory.iterdir()) if directory.exists() else set()
        self.assertEqual(run('--dry-run').returncode, 0)
        env = dict(os.environ, PATH='')
        result = run('--tier', 'Change', '--name', 'runner', env=env)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('cargo missing', result.stderr)
        self.assertEqual(before, set(directory.iterdir()) if directory.exists() else set())


    def test_failed_workload_fails_run_and_records_status(self):
        scratch = ROOT / '.work' / 'tmp'
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as tools:
            tools = pathlib.Path(tools)
            for name in ('cargo', 'rustc'):
                path = tools / name
                path.write_text('#!/bin/sh\nif [ "$1" = "--version" ]; then echo test; exit 0; fi\nexit 7\n')
                path.chmod(0o755)
            (tools / 'git').symlink_to(shutil.which('git'))
            result = run('--name', 'accounting', env=dict(os.environ, PATH=str(tools)))
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertIn('accounting: exit 7', result.stdout)
            summary = pathlib.Path(result.stdout.split('summary: ')[1].strip())
            try:
                self.assertIn('accounting: exit 7', summary.read_text())
            finally:
                shutil.rmtree(summary.parent)


if __name__ == '__main__':
    unittest.main()
