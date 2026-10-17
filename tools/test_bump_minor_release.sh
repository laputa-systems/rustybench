#!/bin/sh

set -u
LC_ALL=C
export LC_ALL

TEST_DIR=$(CDPATH= cd -P "$(dirname "$0")" && pwd -P) || exit 1
SCRIPT=$TEST_DIR/bump_minor_release.sh

failures=0
checks=0

extract_function() {
    sed -n "/^$1() {/,/^}$/p" "$SCRIPT"
}

load_function() {
    definition=$(extract_function "$1")
    if [ -z "$definition" ]; then
        printf '%s\n' "cannot load $1 from $SCRIPT" >&2
        return 1
    fi
    eval "$definition"
}

check_comparison() {
    expected=$1
    published=$2
    target=$3
    checks=$((checks + 1))
    if version_at_least "$published" "$target"; then
        actual=at_least
    else
        actual=below
    fi
    if [ "$actual" != "$expected" ]; then
        printf '%s\n' "FAIL version_at_least $published $target: expected $expected, got $actual" >&2
        failures=$((failures + 1))
    fi
}

check_publish() {
    expected=$1
    published=$2
    target=$3
    checks=$((checks + 1))
    if target_already_published "$published" "$target"; then
        actual=skip
    else
        actual=publish
    fi
    if [ "$actual" != "$expected" ]; then
        printf '%s\n' "FAIL target_already_published $published $target: expected $expected, got $actual" >&2
        failures=$((failures + 1))
    fi
}

check_prerelease() {
    expected=$1
    version=$2
    checks=$((checks + 1))
    if version_is_prerelease "$version"; then
        actual=yes
    else
        actual=no
    fi
    if [ "$actual" != "$expected" ]; then
        printf '%s\n' "FAIL version_is_prerelease $version: expected $expected, got $actual" >&2
        failures=$((failures + 1))
    fi
}

if load_function version_at_least; then
    check_comparison at_least 1.4.0 1.4.0
    check_comparison at_least 1.5.0 1.4.0
    check_comparison below 1.3.9 1.4.0
    check_comparison below 1.4.0 1.5.0
    check_comparison at_least 1.10.0 1.9.0
    check_comparison below 1.9.0 1.10.0
    check_comparison at_least 1.4.0+build.2 1.4.0
    check_comparison at_least 1.4.0 1.4.0-rc.1
    check_comparison below 1.4.0-rc.1 1.4.0
    check_comparison at_least 1.4.0-rc.2 1.4.0-rc.1
    check_comparison below 1.4.0-rc.1 1.4.0-rc.2
    check_comparison at_least 1.4.0-rc.10 1.4.0-rc.9
    check_comparison below 1.4.0-rc.2 1.4.0-rc.10
    check_comparison below 1.4.0-alpha 1.4.0-alpha.1
    check_comparison at_least 1.4.0-alpha.1 1.4.0-alpha
    check_comparison at_least 1.4.0-alpha.beta 1.4.0-alpha.1
else
    printf '%s\n' "cannot check version_at_least: missing from $SCRIPT" >&2
    failures=$((failures + 1))
fi

if load_function version_is_prerelease; then
    check_prerelease yes 1.4.0-rc.1
    check_prerelease no 1.4.0
    check_prerelease no 1.4.0+build-1
    check_prerelease yes 1.4.0-rc.1+build-2
else
    printf '%s\n' "cannot check version_is_prerelease: missing from $SCRIPT" >&2
    failures=$((failures + 1))
fi

if load_function target_already_published; then
    check_publish skip 1.4.0 1.4.0
    check_publish skip 1.5.0 1.4.0
    check_publish publish 1.3.9 1.4.0
    check_publish publish 1.4.0 1.5.0
    check_publish skip 1.4.0+build.7 1.4.0
    check_publish publish 1.4.0-rc.1 1.4.0
    check_publish skip 1.4.0-rc.1 1.4.0-rc.1
    check_publish publish 1.4.0-rc.2 1.4.0-rc.1
    check_publish publish 2.0.0-alpha.1 1.9.9
else
    printf '%s\n' "cannot check target_already_published: missing from $SCRIPT" >&2
    failures=$((failures + 1))
fi

if [ "$failures" -ne 0 ]; then
    printf '%s\n' "$failures of $checks checks failed" >&2
    exit 1
fi

printf '%s\n' "$checks checks passed"
