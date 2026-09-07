#!/usr/bin/env bash
# Run only after the coordinator confirms the remote compiler is ready:
# ssh -p 19987 root@74.2.96.42 bash /root/ecdsafail-cpu/main/remote-run.sh \
#   /root/ecdsafail-cpu/coeff bash -s < this-file
set -euo pipefail
test "$PWD" = /root/ecdsafail-cpu/coeff
test "$(uname -s)" = Linux
baseline=/root/ecdsafail-cpu/baseline
logs="$PWD/validation/narrow-coefficients"
mkdir -p "$logs"
export MIDQ_NARROW_COEFFICIENTS=1 MIDQ_PARK_CHECKPOINT_SELECTORS=1
export MIDQ_OUTER_VENT_QCAP=1009 MIDQ_PZ_VENT_QCAP=1009
export MIDQ_PREFIX_QCAP=1009 MIDQ_CHUNK_COMPARE_QCAP=1009
export MIDQ_CHUNKED_PREFIX=1 MIDQ_CHUNK_COMPARE=1
unset COMMON_SEED_OPS

cargo build --release --bin count_tof --bin build_circuit --bin eval_common > "$logs/build.log" 2>&1
MIDQ_NARROW_COEFFICIENT_SELFTEST=1 target/release/count_tof > "$logs/components.log" 2>&1
MIDQ_COUNTER_TAPE_SELFTEST=1 target/release/count_tof > "$logs/full-tail.log" 2>&1
MIDQ_TRACE_TAIL=1 TRACE_PHASE_ACTIVE=1 TRACE_PHASE_ACTIVE_TOP=40 \
  target/release/build_circuit > "$logs/generation.log" 2>&1
sha256sum ops.bin "$baseline/ops.bin" > "$logs/ops-sha256.txt"
COMMON_SEED_OPS="$baseline/ops.bin" target/release/eval_common \
  --note 'narrow coefficients and parked selectors, frozen verified Q1019 inputs' > "$logs/frozen-9024.log" 2>&1

# Nonzero correctness exits are expected on the independent corpus. Preserve
# those reports, then explicitly compare failure support instead of accepting
# a failed process as a successful test.
candidate_status=0
target/release/eval_common --note 'narrow coefficients, independent 9024' \
  > "$logs/independent-candidate.log" 2>&1 || candidate_status=$?
baseline_status=0
(cd "$baseline" && /root/ecdsafail-cpu/coeff/target/release/eval_common \
  --note 'verified Q1019, paired independent 9024') \
  > "$logs/independent-baseline.log" 2>&1 || baseline_status=$?
test "$candidate_status" -le 1
test "$baseline_status" -le 1

python3 - "$logs" <<'PY'
import pathlib
import re
import sys

logs = pathlib.Path(sys.argv[1])

def parse(name):
    text = (logs / name).read_text()
    assert re.search(r"tested shots\s*:\s*9024\b", text), name
    classical = set(map(int, re.findall(r"COMMON_CLASSICAL_FAILURE (\d+)", text)))
    phase = set()
    for batch, mask in re.findall(r"COMMON_PHASE_FAILURE batch=(\d+) mask=([0-9a-f]+)", text):
        bits = int(mask, 16)
        phase.update(64 * int(batch) + i for i in range(64) if bits >> i & 1)
    ancilla = re.search(r"ancilla-garbage batches\s*:\s*(\d+)", text)
    assert ancilla and int(ancilla[1]) == 0, (name, "ancilla failures")
    return classical, phase

old_classical, old_phase = parse("independent-baseline.log")
new_classical, new_phase = parse("independent-candidate.log")
assert new_classical == old_classical, ("classical failure IDs changed", old_classical, new_classical)
assert new_classical | new_phase == old_classical | old_phase, ("any-channel failure IDs changed", new_phase, old_phase)
print("Paired independent 9024: identical classical and any-channel failure support, no ancilla failures.")
print("Classical failure IDs:", sorted(new_classical))
print("Candidate phase failure IDs:", sorted(new_phase))
PY
printf 'Validation complete; logs: %s\n' "$logs"
