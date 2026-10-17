# Backlog campaign handoff

The campaign is complete on branch `backlog-2026-10-17` through commit `92195f8`. `TODO.md` has no open tickets and keeps `Next: 23`. Nothing was pushed.

## Retained worktrees

The lane checkouts below are clean. Their heads are not ancestors of the campaign head, and `git cherry` reports an unmatched commit for each lane. The cleanup check could not prove patch equivalence, so the checkouts and branches remain for review.

| Checkout | Branch | Head |
| --- | --- | --- |
| `.work/wt/RUST-7` | `lane/RUST-7` | `fef19ac` |
| `.work/wt/RUST-8` | `lane/RUST-8` | `902eddd` |
| `.work/wt/RUST-9` | `lane/RUST-9` | `41ebd0b` |
| `.work/wt/RUST-10` | `lane/RUST-10` | `266b3be` |
| `.work/wt/RUST-11` | `lane/RUST-11` | `891f9dd` |
| `.work/wt/RUST-12` | `lane/RUST-12` | `0089667` |
| `.work/wt/RUST-14` | `lane/RUST-14` | `b5ce314` |
| `.work/wt/RUST-16` | `lane/RUST-16` | `87839d1` |
| `.work/wt/RUST-17` | `lane/RUST-17` | `3b954c8` |
| `.work/wt/RUST-18` | `lane/RUST-18` | `e19c6dc` |
| `.work/wt/RUST-19` | `lane/RUST-19` | `e434379` |
| `.work/wt/RUST-20` | `lane/RUST-20` | `68d4778` |
| `.work/wt/RUST-21` | `lane/RUST-21` | `9fe9ae4` |
| `.work/wt/RUST-22` | `lane/RUST-22` | `50e9af6` |

`.work/tmp/prebase` remains outside the campaign lanes. Its clean checkout is at `757f502` on `prebase/8-9`. It has two commits not in the campaign branch, so it remains for inspection.
