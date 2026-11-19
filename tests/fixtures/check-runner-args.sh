#!/bin/sh
set -eu

fixture_dir=${0%/*}
IFS= read -r expected < "$fixture_dir/arguments"
[ "$*" = "$expected" ]
cat "$fixture_dir/report.json"
