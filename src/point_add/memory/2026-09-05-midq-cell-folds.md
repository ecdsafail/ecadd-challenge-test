# Exact carry networks in the ping-pong coefficient cell

Base: `72190cc`, verified Q1019, average T 14,748,731.513 on the
baseline's own 9,024 Fiat-Shamir inputs. Frozen `ops.bin` SHA256:
`403563ee39701de8af489f680fc4efd0f26afd97dcc4ab4db738385aae74908f`.

## Scope and switches

`MIDQ_CELL_FOLDS=1` dispatches only `midq_mod_signed_add_halve` to the new
`cell_folds` helper. The default is off. `MIDQ_CELL_QCAP` defaults to 1009
and is limited to at most 1019. `MIDQ_CELL_SUM=0` disables the integer-sum
replacement while retaining the correction replacements.

The arithmetic operations, operand widths, sign handling, rotations, comparison
and overflow cleanup are unchanged. Forward and inverse cells are both covered.
This is a carry-network improvement, not a claim that the two modular corrections
have been combined algebraically. The existing comparison is evaluated on the
post-fold value and can have a nonzero error phase for noncanonical inputs.
Replacing its predicate by a pre-fold comparison without retaining that behavior
would not be an exact rewrite of the baseline channel.

No changes to PZ rounds, tail rounds, operand-width schedules, guards, coefficient
layout, sign-bit encoding, or trusted harness files are included.

## Carry identity

Let a bit column contain target a, addend b, and incoming carry c. Its updated sum
is s = a XOR b XOR c, and outgoing carry d = MAJ(a,b,c). On all eight inputs,
also d = MAJ(NOT s,b,c). No canonical-field or arithmetic-range assumption enters.

One clean outgoing carry costs one CCX via
`d = c XOR ((a XOR c) AND (b XOR c))`. The target and addend are restored around
this computation before the sum bit is installed. For a constant-zero column the
carry reduces to a AND c. For the first column the incoming carry is zero.

After the sum update, HMR of d introduces phase m*d. Under m, the three CZ terms
`(NOT s)*b`, `(NOT s)*c`, and `b*c` cancel it exactly. Constant-zero terms are
omitted. Thus all internal carries can be discharged without Toffolis as long as
their incoming carries remain live.

Bounded chunks retain their outgoing boundary carries. After all sum bits are
written, boundaries are cleared right-to-left. Their local carry chain is
recomputed from the updated sum using MAJ(NOT s,b,c), only under the boundary
measurement bit; the final majority is applied as a phase. Local replay carries
are cleared by the same phase identity. This works for every measurement branch.

The shared `clean_chunk_plan.rs` is imported from the main optimization worktree,
not independently redesigned here. With C carry columns and A available clean
wires, its feasible plans use at most A scratch wires and C-A local replay
columns (clamped at zero). For nonzero initial columns the typical cost is C
forward CCX plus one half of the replay CCX on average. If no plan fits, the
original primitive runs unchanged. Both the F correction and the integer sum
use this planner; each call rechecks its actual live-register count.

For signed subtraction, the original complement-add-complement frames remain.
For an F subtraction the target is complemented around the exact F adder.
The low-256-bit addition is modulo 2^256 on every word. The sum primitive XORs
its carry into the original overflow wire, including when that wire starts at 1.
The original comparison HMR and phase oracle are retained without modification.

## Validation

Component hook: `MIDQ_CELL_FOLDS_SELFTEST=1 target/release/count_tof`.
Tests use the unmodified `crate::sim::Simulator`, plus an independent flattening
of nested classical conditions. Every R operation is checked for zero before
execution, not merely after reset. Native and flattened values, phases, and gate
statistics must agree. Donor/input registers remain arbitrary and must be restored.

The constant-adder tests enumerate every constant and input/control combination
at widths 1 through 7, both add and subtract, and each uniform chunk size. The
sum tests enumerate both inputs and an arbitrary initial overflow bit. They use
all-zero, all-one, and random measurement streams, and enumerate all measurement
transcripts for widths through 3. Full-width old/new comparisons include both
directions, arbitrary 257th input wires, random inputs, long carries, and field
boundary values. Their original comparison measurements are coupled so inherited
noncanonical error phases are compared, rather than incorrectly requiring zero.

The uniform-chunk prototype passed 2,712,896 component cases. Its correction-only
whole-circuit variant passed all frozen 9,024 inputs with 0/0/0 errors at Q1019 and
T=14,466,905.642, a saving of 281,825.871 average Toffolis. That prototype is not
the final unequal-chunk variant.

The uniform-chunk variant with the integer-sum replacement also passed all frozen
9,024 inputs with 0/0/0 errors at Q1019 and T=14,415,324.827, saving 333,406.686
average Toffolis. The unequal-chunk version passed the original 2,712,896-case
component suite and generated 87,208,400 operations before the local-work stop.
The expanded field-edge component cases and full unequal-chunk regressions still
require remote execution. No latest-variant resource claim is made yet.

The final remote unequal-chunk frozen test passed all 9,024 inputs with 0/0/0
errors at Q1019 and average T=14,228,327.568. Its total Toffoli count was
128,396,427,973 over 9,024 shots; average Clifford count was 38,979,102.910.
This is a 520,403.945 average-Toffoli reduction from the verified baseline.
The artifact contains 87,208,400 operations and has SHA256
`d3d5120e37bf498ca06cb9b4516ac2894f1490ce87a3dbfae8961dfe58da3b75`.
Before the broader constant experiment, it was preserved at
`/root/ecdsafail-cpu/field/validation/preserved-ops.Nls44O/ops.bin`.
The remote manifest hashes match the local carry module, component tests, state
machine, shared planner, and diagnostic evaluator sources.

The requested Q<=1009/T<12.5M milestone is not claimed by this isolated branch;
unrelated baseline phases can still peak at 1019. The completed remote component
suite passed 2,725,184 cases plus the 263,169-schedule planner audit. The resumed
independent 9,024-input evaluation retained exactly the five classical failure
inputs {4174, 4968, 5407, 7617, 7854}, with no phase or ancilla failures in that
measurement run. Its any-channel failing-input set therefore equals the baseline
set. This is frozen-input regression validation, not a clean Fiat-Shamir nonce
for the new op stream; no submission was made.

## Remote-only continuation

The user permanently prohibited heavy local builds, generation, and evaluation
because the laptop became unresponsive. All local field jobs have exited. Wait
for coordination before launching remote work. The allocation is 16 vCPU, one
RTX5090, and 32 GB RAM, regardless of host-wide readings. The remote source is
`/root/ecdsafail-cpu/field`; the wrapper
`/root/ecdsafail-cpu/main/remote-run.sh` controls toolchain, PATH, build jobs (3),
and the global `/root/ecdsafail-cpu/heavy-job.lock` flock. All heavy jobs must
pass through this wrapper, without a nested lock or parallel GPU nonce work.

After copying the newly added `validate-field-remote.sh`, invoke it through that
wrapper. The runner refuses a different working directory, verifies the frozen
baseline hash, runs jobs serially, and preserves separate logs. An independent
corpus is expected to fail on the five inherited inputs. The runner checks the
exact classical failure set, no ancilla failures, and that every phase-failure
input remains within that same set. Code should be committed only after these
remote checks complete successfully.

Receipts for this run are under
`/root/ecdsafail-cpu/field/validation/field.gnsjYJ`. The first independent
evaluation was terminated when the resource allocation was corrected. Its
partial log is preserved. The runner's `resume-independent <receipt-directory>`
mode verifies source and artifact hashes, reuses completed checks, and restarts
only the interrupted evaluation under the global wrapper.
