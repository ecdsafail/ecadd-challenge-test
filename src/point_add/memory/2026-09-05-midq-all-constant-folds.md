# Broader exact bounded constant folds

Separate follow-up to validated cell commit `de20fb5` on base `72190cc`.
`MIDQ_ALL_CONST_FOLDS=1` additionally routes eligible calls in
`midq_constant_update` and the outer `controlled_rfold_window` through the same
bounded constant adder. The switch is off by default. `MIDQ_CELL_QCAP=1009`
applies to both cell and broader constant work; a failed headroom check retains
the original adder. There are no changes to arithmetic widths, guards, rounds,
or approximation parameters.

## Reusable primitive

`cell_folds::try_constant_update(c, ctrl, target, constant, subtract)` implements
the original word permutation `target +=/-= ctrl*constant mod 2^target.len()`.
It accepts arbitrary constant bytes and does not require canonical field values
or a zero top bit. It changes no donor register. The phase proof and bounded
replay schedule are exactly those recorded in the preceding cell note. Control
aliasing the target is rejected, and callers leave the original route in place
when the control aliases its borrowed-donor region.

The outer call deliberately passes only its existing 73-bit window. It does not
propagate any new carry into bit 73 or change the inherited window truncation.
Normalization and field negation pass their original whole words and constants
p or p+1 unchanged. In particular, the helper also accepts 256-bit slices used
by the separately developed coefficient-layout reduction.

This extension leaves `cell_folds::signed_add` unchanged. The main worker owns
the temporary overflow-wire adaptation needed when that function receives a
256-bit target. Only `fold` is refactored to use the reusable helper.

## Validation plan

The remote-only `profile-all-constants-remote.sh` builds the diagnostic binary,
reruns the existing cell component tests, and costs the new real call sites.
The new hook is `MIDQ_ALL_CONST_FOLDS_SELFTEST=1`. Its oracle is an independent
bit-plane addition/subtraction recurrence over the complete target word. Cases
include p, p+1, F, a 257-bit arbitrary constant, the real 73-bit outer window,
arbitrary input bit 256, random donors and spectators, long carries, zero and
all-one inputs, add/subtract, and several live-register budgets.

Old and new implementations are both required to match that oracle, with zero
phase and no dirty resets, under zero, one, and random measurement streams.
No-headroom calls must emit byte-identical fallback streams. Component costs
must be reported before any new whole-circuit generation. All heavy work runs
through the main remote wrapper's single global flock under the user's fixed
16-vCPU, one-RTX5090, 32-GB allocation. No heavy local work is permitted.

## Completed component costs

Remote receipt directory:
`/root/ecdsafail-cpu/field/validation/all-constants-cost.3eiSFr`.
The extension passed 307,200 full-width oracle cases. The original cell suite
also passed all 2,725,184 cases with the refactored helper.

Measured mean Toffolis (1,024 random-measurement shots per resource row):

| Primitive | Live wires before call | Old T | New T | New Q |
| --- | ---: | ---: | ---: | ---: |
| 257-bit add p | 560 | 767 | 256.000 | 816 |
| 257-bit add p | 980 | 767 | 365.547 | 1009 |
| 257-bit add p+1 | 980 | 767 | 363.953 | 1009 |
| 257-bit add p | 995 | 767 | 767.000 | 997 |
| 73-bit outer F fold | 560 | 215 | 72.000 | 632 |
| 73-bit outer F fold | 980 | 215 | 92.359 | 1009 |
| 73-bit outer F fold | 995 | 215 | 100.219 | 1009 |

The 995-live-wire 257-bit call falls back byte-identically. Local primitive Q
can increase within the 1009 cap; these are not claims of a lower per-call
space requirement. All production aggregate claims await whole-circuit results.
The component costs were reported before queuing whole-circuit generation.

## Completed whole-circuit validation

Remote receipt directory: `/root/ecdsafail-cpu/field/validation/field.MKrtyk`.
Exact generation flags: `MIDQ_CELL_FOLDS=1 MIDQ_CELL_SUM=1
MIDQ_ALL_CONST_FOLDS=1 MIDQ_CELL_QCAP=1009`, with the remaining defaults inherited
from `de20fb5` / `72190cc`. All heavy commands ran serially through the wrapper.

- Q = 1019.
- Frozen 9,024 inputs: 0 classical, 0 phase, 0 ancilla failures.
- Average T = 13,723,510.967; total T = 123,840,962,965 over 9,024 shots.
- Average Clifford count = 38,355,043.838.
- Emitted operations = 87,202,574.
- Marginal average-T saving from `de20fb5` = 504,816.601.
- Total average-T saving from the verified original baseline = 1,025,220.546.
- Independent 9,024 inputs: exactly the original classical failures
  {4174, 4968, 5407, 7617, 7854}; phase failure only at 4174; no ancilla failures.
  Hence the any-channel failing-input set is unchanged.

Generated `ops.bin` SHA256:
`fdac30f512b79a785390053a9e50fea5ce0c2b936fd120190c41c9e48ac33ac4`.
The preceding `de20fb5` op stream was preserved before generation at
`/root/ecdsafail-cpu/field/validation/preserved-ops.Nls44O/ops.bin`.
Completed component receipts were reused after a source-hash check; they were
not rerun or combined with another heavy job. This is a frozen-input regression
result, not a clean Fiat-Shamir nonce for the new stream. No submission was made.

The isolated branch still has Q1019 peaks outside these helpers. It does not
independently meet the main target Q<=1009/T<12.5M. The source is deliberately
separate from the main worker's 256-bit coefficient and controlled-word-adder
changes, which require their own integrated resource and correctness checks.
