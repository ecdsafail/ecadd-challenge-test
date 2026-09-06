#!/usr/bin/env bash
# Run only through the coordinated remote wrapper after compiler readiness.
set -euo pipefail
case "$PWD" in
  /root/ecdsafail-cpu/field) ;;
  *) printf 'Refusing heavy work outside /root/ecdsafail-cpu/field\n' >&2; exit 2 ;;
esac

baseline=/root/ecdsafail-cpu/baseline/ops.bin
expected=403563ee39701de8af489f680fc4efd0f26afd97dcc4ab4db738385aae74908f
printf '%s  %s\n' "$expected" "$baseline" | sha256sum --check --status
mkdir -p validation
if [[ "${1:-}" == resume-independent ]]; then
  logdir=${2:?Existing validation directory required}
  case "$logdir" in "$PWD"/validation/field.*) ;; *) exit 2 ;; esac
  sha256sum --check --status "$logdir/sources.sha256" "$logdir/ops.sha256"
  grep -Fq 'MIDQ_CELL_FOLDS_SELFTEST PASS:' "$logdir/components.log"
  grep -Fq '=== build_circuit OK ===' "$logdir/generate.log"
  grep -Fq '=== experiment OK ===' "$logdir/frozen.log"
  if [[ -e "$logdir/independent.log" ]]; then
    interrupted=$(mktemp "$logdir/independent.interrupted.XXXXXX.log")
    mv "$logdir/independent.log" "$interrupted"
  fi
elif [[ "${1:-}" == from-component-cost ]]; then
  costdir=${2:?Completed component-cost directory required}
  case "$costdir" in "$PWD"/validation/all-constants-cost.*) ;; *) exit 2 ;; esac
  sha256sum --check --status "$costdir/sources.sha256"
  grep -Fq 'MIDQ_CELL_FOLDS_SELFTEST PASS:' "$costdir/cell-regression.log"
  grep -Fq 'MIDQ_ALL_CONST_FOLDS_SELFTEST PASS:' "$costdir/components.log"
  logdir=$(mktemp -d "$PWD/validation/field.XXXXXX")
  cp "$costdir/sources.sha256" "$logdir/sources.sha256"
  cp "$costdir/cell-regression.log" "$logdir/components.log"
  cp "$costdir/components.log" "$logdir/global-constants.log"
  printf '%s\n' 'MIDQ_CELL_FOLDS=1 MIDQ_CELL_SUM=1 MIDQ_ALL_CONST_FOLDS=1 MIDQ_CELL_QCAP=1009' > "$logdir/flags.txt"
  if [[ -e ops.bin ]]; then
    preserved=$(mktemp -d "$PWD/validation/preserved-ops.XXXXXX")
    mv ops.bin "$preserved/ops.bin"
    printf '%s\n' "$preserved/ops.bin" > "$logdir/previous-ops.txt"
  fi
  cargo build --release --bin build_circuit --bin eval_common \
    > "$logdir/build.log" 2>&1
  MIDQ_CELL_FOLDS=1 MIDQ_CELL_SUM=1 MIDQ_ALL_CONST_FOLDS=1 MIDQ_CELL_QCAP=1009 \
    target/release/build_circuit > "$logdir/generate.log" 2>&1
  sha256sum ops.bin > "$logdir/ops.sha256"
  COMMON_SEED_OPS="$baseline" target/release/eval_common \
    > "$logdir/frozen.log" 2>&1
else
  [[ $# -eq 0 ]] || exit 2
  logdir=$(mktemp -d "$PWD/validation/field.XXXXXX")
  sha256sum src/point_add/clean_chunk_plan.rs \
    src/point_add/trailmix_port/inversion/cell_folds*.rs \
    src/point_add/trailmix_port/inversion/shrunken_pz_state_machine.rs \
    src/bin/eval_common.rs > "$logdir/sources.sha256"

  cargo build --release --bin count_tof --bin build_circuit --bin eval_common \
    > "$logdir/build.log" 2>&1
  MIDQ_CELL_FOLDS_SELFTEST=1 target/release/count_tof \
    > "$logdir/components.log" 2>&1
  MIDQ_CELL_FOLDS=1 MIDQ_CELL_SUM=1 MIDQ_CELL_QCAP=1009 \
    target/release/build_circuit > "$logdir/generate.log" 2>&1
  sha256sum ops.bin > "$logdir/ops.sha256"
  COMMON_SEED_OPS="$baseline" target/release/eval_common \
    > "$logdir/frozen.log" 2>&1
fi
printf 'Field validation logs: %s\n' "$logdir"

status=0
env -u COMMON_SEED_OPS target/release/eval_common \
  > "$logdir/independent.log" 2>&1 || status=$?
printf '%s\n' "$status" > "$logdir/independent.exit"
python3 - "$logdir" "$status" > "$logdir/summary.txt" <<'PY'
import pathlib
import re
import sys

logs = pathlib.Path(sys.argv[1])
status = int(sys.argv[2])
frozen = (logs / "frozen.log").read_text()
independent = (logs / "independent.log").read_text()
expected = {4174, 4968, 5407, 7617, 7854}

def metric(text, label):
    found = re.search(r"^\s*" + re.escape(label) + r"\s*:\s*(\d+)\s*$", text, re.M)
    assert found, f"Missing completed metric: {label}"
    return int(found[1])

assert metric(frozen, "tested shots") == 9024
for label in ("classical mismatches", "phase-garbage batches", "ancilla-garbage batches"):
    assert metric(frozen, label) == 0, label
assert status == 1, f"Expected inherited independent failures, got exit {status}"
assert metric(independent, "tested shots") == 9024
assert metric(independent, "classical mismatches") == len(expected)
assert metric(independent, "ancilla-garbage batches") == 0
classical = {int(i) for i in re.findall(r"^COMMON_CLASSICAL_FAILURE (\d+)$", independent, re.M)}
assert classical == expected, (classical, expected)
phase_rows = re.findall(r"^COMMON_PHASE_FAILURE batch=(\d+) mask=([0-9a-f]+)$", independent, re.M)
assert metric(independent, "phase-garbage batches") == len(phase_rows)
phase = {int(batch) * 64 + bit for batch, mask in phase_rows
         for bit in range(64) if (int(mask, 16) >> bit) & 1}
assert phase <= expected, f"New any-channel failure inputs: {sorted(phase - expected)}"
print("PASS: frozen 9024 = 0/0/0; independent classical/any-channel failure set unchanged.")
print(f"Independent phase-failure inputs: {sorted(phase)}")
print(frozen[frozen.index("=== circuit metrics"):].strip())
PY
grep -F 'MIDQ_CELL_FOLDS_SELFTEST PASS:' "$logdir/components.log"
cat "$logdir/summary.txt"
printf 'All field checks completed. Receipts: %s\n' "$logdir"
