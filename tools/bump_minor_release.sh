#!/bin/sh

set -eu
LC_ALL=C
export LC_ALL

fail() {
    printf '%s\n' "make bump: $*" >&2
    exit 1
}

SCRIPT_DIR=$(CDPATH= cd -P "$(dirname "$0")" && pwd -P) || fail "cannot locate script directory"
ROOT=$(CDPATH= cd -P "$SCRIPT_DIR/.." && pwd -P) || fail "cannot locate repository root"
MANIFEST=$ROOT/Cargo.toml
MACROS_MANIFEST=$ROOT/macros/Cargo.toml
LOCKFILE=$ROOT/Cargo.lock
cd "$ROOT"

for command_name in awk cargo cp dirname git mkdir mv rm sed; do
    command -v "$command_name" >/dev/null 2>&1 || fail "required command not found: $command_name"
done

temporary_manifest=
backup_dir=
release_started=0
release_committed=0
release_subject=

on_exit() {
    exit_status=$1
    trap - 0 HUP INT TERM

    if [ "$release_started" -eq 1 ] && [ "$release_committed" -eq 0 ]; then
        if head_subject=$(git log -1 --format=%s 2>/dev/null); then
            :
        else
            head_subject=
        fi
        if [ "$head_subject" != "$release_subject" ]; then
            if ! git reset -- Cargo.toml Cargo.lock macros/Cargo.toml; then
                printf '%s\n' "make bump: failed to unstage release inputs; backups remain in $backup_dir" >&2
                exit 1
            fi
            if ! cp -p "$backup_dir/Cargo.toml" "$MANIFEST" ||
                ! cp -p "$backup_dir/Cargo.lock" "$LOCKFILE" ||
                ! cp -p "$backup_dir/macros-Cargo.toml" "$MACROS_MANIFEST"; then
                printf '%s\n' "make bump: failed to restore release inputs; backups remain in $backup_dir" >&2
                exit 1
            fi
        fi
    fi

    if [ -n "$temporary_manifest" ]; then
        rm -f "$temporary_manifest" || exit_status=1
    fi
    if [ -n "$backup_dir" ]; then
        rm -rf "$backup_dir" || exit_status=1
    fi
    exit "$exit_status"
}

trap 'on_exit $?' 0
trap 'exit 1' HUP INT TERM

read_package_metadata() {
    awk '
        /^[ \t]*\[package\][ \t]*(#.*)?$/ { section = "package"; next }
        /^[ \t]*\[/ { section = ""; next }
        section == "package" {
            line = $0
            sub(/^[ \t]*/, "", line)
            key = line
            sub(/[ \t]*=.*/, "", key)
            if (key != "name" && key != "version") next

            value = line
            sub(/^[^=]*=[ \t]*/, "", value)
            if (key == "name") {
                name_count++
                if (value !~ /^"[-A-Za-z0-9_]+"([ \t]*#.*)?$/) invalid = 1
                sub(/^"/, "", value)
                sub(/".*/, "", value)
                package_name = value
            } else {
                version_count++
                if (value !~ /^"(0|[1-9][0-9]*)[.](0|[1-9][0-9]*)[.](0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?([+][0-9A-Za-z.-]+)?"([ \t]*#.*)?$/) invalid = 1
                sub(/^"/, "", value)
                sub(/".*/, "", value)
                package_version = value
            }
        }
        END {
            if (invalid || name_count != 1 || version_count != 1) exit 1
            print package_name
            print package_version
        }
    ' "$1"
}

package_metadata=$(read_package_metadata "$MANIFEST") || fail "expected literal name and semantic version fields in the root [package] section"

package_name=$(printf '%s\n' "$package_metadata" | sed -n '1p')
current_version=$(printf '%s\n' "$package_metadata" | sed -n '2p')
[ -n "$package_name" ] && [ -n "$current_version" ] || fail "root package metadata is incomplete"

# The macros member tracks the runner version exactly; the runner pins it
# with an exact version requirement, so both crates release in lockstep.
macros_metadata=$(read_package_metadata "$MACROS_MANIFEST") || fail "expected literal name and semantic version fields in the macros [package] section"
macros_name=$(printf '%s\n' "$macros_metadata" | sed -n '1p')
macros_version=$(printf '%s\n' "$macros_metadata" | sed -n '2p')
[ -n "$macros_name" ] && [ -n "$macros_version" ] || fail "macros package metadata is incomplete"
[ "$macros_version" = "$current_version" ] || fail "workspace member versions must match: $package_name is $current_version but $macros_name is $macros_version"

# Cargo checks registry ownership before the script edits any release file.
cargo owner --list "$package_name"
cargo owner --list "$macros_name"

registry_version() {
    search_result=$(cargo search "$1" --limit 1) || fail "could not query the default Cargo registry"
    printf '%s\n' "$search_result" | awk -v package_name="$1" '
        $1 == package_name && $2 == "=" && $3 ~ /^".+"$/ {
            version = substr($3, 2, length($3) - 2)
            matches++
        }
        END {
            if (matches != 1) exit 1
            print version
        }
    ' || fail "could not find $1 on the default Cargo registry"
}

version_at_least() {
    awk -v left="$1" -v right="$2" '
        function components(value, result) {
            sub(/[+].*$/, "", value)
            sub(/-.*/, "", value)
            return split(value, result, "[.]")
        }
        function compare_component(left_part, right_part) {
            if (length(left_part) < length(right_part)) return -1
            if (length(left_part) > length(right_part)) return 1
            if (("x" left_part) < ("x" right_part)) return -1
            if (("x" left_part) > ("x" right_part)) return 1
            return 0
        }
        BEGIN {
            components(left, left_parts)
            components(right, right_parts)
            for (part = 1; part <= 3; part++) {
                comparison = compare_component(left_parts[part], right_parts[part])
                if (comparison != 0) exit (comparison > 0 ? 0 : 1)
            }
            exit 0
        }
    '
}

publish_package() {
    already_published=$(registry_version "$1") || exit 1
    if version_at_least "$already_published" "$2"; then
        printf '%s\n' "$1 $2 is already published; skipping"
        return 0
    fi
    cargo publish -p "$1"
}

publish_workspace() {
    # Dependency order matters: the runner pins an exact macros version,
    # so the macros crate must reach the registry first. Each crate is
    # skipped when already published, so re-running after a partial
    # publish finishes the remaining crates instead of failing.
    publish_package "$macros_name" "$1"
    publish_package "$package_name" "$1"
}

latest_version=$(registry_version "$package_name")

version_plan=$(awk -v current="$current_version" -v latest="$latest_version" '
    function valid_version(value) {
        return value ~ /^(0|[1-9][0-9]*)[.](0|[1-9][0-9]*)[.](0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?([+][0-9A-Za-z.-]+)?$/
    }
    function components(value, result) {
        sub(/[+].*$/, "", value)
        sub(/-.*/, "", value)
        return split(value, result, "[.]")
    }
    function compare_component(left, right) {
        if (length(left) < length(right)) return -1
        if (length(left) > length(right)) return 1
        if (("x" left) < ("x" right)) return -1
        if (("x" left) > ("x" right)) return 1
        return 0
    }
    function compare_versions(left, right, left_parts, right_parts, part, comparison) {
        components(left, left_parts)
        components(right, right_parts)
        for (part = 1; part <= 3; part++) {
            comparison = compare_component(left_parts[part], right_parts[part])
            if (comparison != 0) return comparison
        }
        return 0
    }
    function increment_decimal(value, result, carry, position, digit, next_digit) {
        result = ""
        carry = 1
        for (position = length(value); position > 0; position--) {
            digit = substr(value, position, 1)
            next_digit = index("0123456789", digit) - 1 + carry
            if (next_digit == 10) {
                next_digit = 0
                carry = 1
            } else {
                carry = 0
            }
            result = substr("0123456789", next_digit + 1, 1) result
        }
        if (carry) result = "1" result
        return result
    }
    BEGIN {
        if (!valid_version(current) || !valid_version(latest)) exit 1
        comparison = compare_versions(current, latest)
        base = current
        if (comparison < 0) base = latest
        components(base, base_parts)
        print comparison
        print base_parts[1] "." increment_decimal(base_parts[2]) ".0"
    }
') || fail "could not parse current or published semantic version"

current_vs_latest=$(printf '%s\n' "$version_plan" | sed -n '1p')
next_version=$(printf '%s\n' "$version_plan" | sed -n '2p')
release_subject="release: $package_name v$next_version"

git_subject=$(git log -1 --format=%s) || fail "could not read the current commit subject"
release_tag=v$current_version
tags_at_head=$(git tag --points-at HEAD) || fail "could not read tags at HEAD"
newline=$(printf '\nX')
newline=${newline%X}

ensure_clean_tree() {
    working_changes=$(git status --porcelain=v1) || fail "could not inspect the working tree"
    [ -z "$working_changes" ] || fail "release requires a clean working tree"
}

if [ "$git_subject" = "release: $package_name v$current_version" ]; then
    ensure_clean_tree
    case "$newline$tags_at_head$newline" in
        *"$newline$release_tag$newline"*) ;;
        *) git tag -a "$release_tag" -m "$package_name $current_version" ;;
    esac
    if [ "$current_vs_latest" -le 0 ]; then
        printf '%s\n' "$package_name $current_version is already published; nothing to do"
        exit 0
    fi
    printf '%s\n' "Retrying publish of $package_name $current_version"
    publish_workspace "$current_version"
    exit 0
fi

[ "$current_vs_latest" -le 0 ] || fail "local version $current_version is newer than published $latest_version, but HEAD is not the matching release commit"
ensure_clean_tree

next_tag=v$next_version
existing_tags=$(git tag) || fail "could not list Git tags"
case "$newline$existing_tags$newline" in
    *"$newline$next_tag$newline"*) fail "tag $next_tag already exists away from the release commit" ;;
esac

backup_dir_candidate=${TMPDIR:-/tmp}/bump_minor_release.$$
(umask 077 && mkdir "$backup_dir_candidate") || fail "could not create private release backups"
backup_dir=$backup_dir_candidate
cp -p "$MANIFEST" "$backup_dir/Cargo.toml" || fail "could not back up Cargo.toml"
cp -p "$LOCKFILE" "$backup_dir/Cargo.lock" || fail "could not back up Cargo.lock"
cp -p "$MACROS_MANIFEST" "$backup_dir/macros-Cargo.toml" || fail "could not back up macros Cargo.toml"
release_started=1

bump_manifest_version() {
    temporary_manifest_candidate=$ROOT/.Cargo.toml.bump.$$
    (umask 077 && set -C && : > "$temporary_manifest_candidate") || fail "could not create temporary manifest"
    temporary_manifest=$temporary_manifest_candidate
    cp -p "$1" "$temporary_manifest" || fail "could not preserve Cargo.toml permissions"
    awk -v old="$2" -v new="$3" '
        /^[ \t]*\[package\][ \t]*(#.*)?$/ { section = "package"; print; next }
        /^[ \t]*\[/ { section = ""; print; next }
        section == "package" {
            line = $0
            sub(/^[ \t]*/, "", line)
            sub(/[ \t]*$/, "", line)
            if (line ~ /^version[ \t]*=/) {
                matches++
                if (line != "version = \"" old "\"") exit 1
                print "version = \"" new "\""
                next
            }
        }
        { print }
        END { if (matches != 1) exit 1 }
    ' "$1" > "$temporary_manifest" || fail "$1 version changed or could not be updated"
    mv "$temporary_manifest" "$1" || fail "could not install updated $1"
    temporary_manifest=
}

bump_manifest_version "$MACROS_MANIFEST" "$current_version" "$next_version"
bump_manifest_version "$MANIFEST" "$current_version" "$next_version"

temporary_manifest_candidate=$ROOT/.Cargo.toml.bump.$$
(umask 077 && set -C && : > "$temporary_manifest_candidate") || fail "could not create temporary manifest"
temporary_manifest=$temporary_manifest_candidate
cp -p "$MANIFEST" "$temporary_manifest" || fail "could not preserve Cargo.toml permissions"
awk -v dep="$macros_name" -v old="$current_version" -v new="$next_version" '
    {
        line = $0
        stripped = line
        sub(/^[ \t]*/, "", stripped)
        if (stripped ~ ("^" dep "[ \t]*=")) {
            matches++
            old_pin = "version = \"=" old "\""
            new_pin = "version = \"=" new "\""
            if (index(line, old_pin) == 0) exit 1
            updated = ""
            rest = line
            replaced = 0
            while ((pos = index(rest, old_pin)) > 0) {
                replaced++
                updated = updated substr(rest, 1, pos - 1) new_pin
                rest = substr(rest, pos + length(old_pin))
            }
            if (replaced != 1) exit 1
            print updated rest
            next
        }
        print
    }
    END { if (matches != 1) exit 1 }
' "$MANIFEST" > "$temporary_manifest" || fail "macros dependency pin changed or could not be updated"
mv "$temporary_manifest" "$MANIFEST" || fail "could not install updated $MANIFEST"
temporary_manifest=

cargo check --workspace --quiet
cargo publish -p "$macros_name" --dry-run --allow-dirty
cargo publish -p "$package_name" --dry-run --allow-dirty
git add -- Cargo.toml Cargo.lock macros/Cargo.toml
git commit -m "$release_subject"
release_committed=1
git tag -a "$next_tag" -m "$package_name $next_version"
publish_workspace "$next_version"
