#!/usr/bin/env python3
"""Replay/reduce every dumped model, retaining all outputs even after a failure."""
import argparse
import os
from pathlib import Path
import subprocess
import sys

p = argparse.ArgumentParser()
p.add_argument('--models', type=Path, required=True)
p.add_argument('--check', required=True)
p.add_argument('--seed', required=True)
a = p.parse_args()
failed = False
for model in sorted(a.models.glob('*.json')):
    proc = subprocess.run([sys.executable, 'scripts/nightly-models.py', '--model', str(model.resolve()),
                           '--check', a.check, '--seed', a.seed, '--seconds', '360', '--reduce-seconds', '120'])
    # Findings deliberately return 1. Other exceptions also fail the collection
    # step; original dumps remain available in the job's separate artifact.
    failed |= proc.returncode != 0
sys.exit(int(failed))
