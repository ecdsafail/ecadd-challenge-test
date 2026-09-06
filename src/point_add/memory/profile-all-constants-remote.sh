#!/usr/bin/env bash
# Component costing only. Invoke through the single global remote wrapper.
set -euo pipefail
[[ "$PWD" == /root/ecdsafail-cpu/field ]] || exit 2
mkdir -p validation
logdir=$(mktemp -d "$PWD/validation/all-constants-cost.XXXXXX")
printf 'Global-constant component receipts: %s\n' "$logdir"
sha256sum src/point_add/clean_chunk_plan.rs src/point_add/mod.rs \
  src/point_add/trailmix_port/inversion/cell_folds*.rs \
  src/point_add/trailmix_port/inversion/all_const_folds_selftest.rs \
  src/point_add/trailmix_port/inversion/shrunken_pz_state_machine.rs \
  src/point_add/trailmix_port/rfold_mbu.rs > "$logdir/sources.sha256"
cargo build --release --bin count_tof > "$logdir/build.log" 2>&1
MIDQ_CELL_FOLDS_SELFTEST=1 MIDQ_ALL_CONST_FOLDS=0 \
  target/release/count_tof > "$logdir/cell-regression.log" 2>&1
MIDQ_ALL_CONST_FOLDS_SELFTEST=1 target/release/count_tof \
  > "$logdir/components.log" 2>&1
grep -F 'MIDQ_CELL_FOLDS_SELFTEST PASS:' "$logdir/cell-regression.log"
awk '/MIDQ_ALL_CONST_FOLDS_SELFTEST PASS/ || (/ALL_CONST_RESOURCE/ && /name=(normalize_p|field_neg_p1|outer_F73) / && /live=(560|980|995) /)' \
  "$logdir/components.log"
printf 'Component costing completed; no whole-circuit generation was run.\n'
