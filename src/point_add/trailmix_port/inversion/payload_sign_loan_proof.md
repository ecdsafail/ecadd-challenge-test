# Read-Only Multiplier High-Slot Sign Loan

Separate, default-off refinement after parity commit
`1eee20bff525806a655f65bcd31fb403ce3be225`.
Enable only with `MIDQ_PAYLOAD_SIGN_LOAN=1`.

## Scope and Exactness

The only changed intervals are the two hybrid outer payload multiplications:
the forward slope computation and the cancellation witness computation.
Coefficients, slope values, input normalization, and the three original
field-negation positions are unchanged. No width, round count, guard, or
approximation parameter is tightened. Generic arithmetic is not modified.

`mod_mul_rfold_mbu(result,a,b)` takes 257-bit vectors. Its first control
is b[255], followed by b[254] through b[0]. Its inverse uses b[0] through
b[254], followed by b[255]. Neither function reads or writes b[256].
Neither passes the full b vector to a callee: each callee receives only
one of those low controls, a, and result. Therefore their operation stream
acts as the identity on b[256], even when that slot contains a quantum
tag entangled with the operands. This is a statement about the existing
approximate circuit itself, not an ideal modular-multiplication identity.
Component tests additionally assert that no emitted operation in either
multiplier direction targets or controls on the high-tag wire.

At these call sites the old b[256] is the canonical zero padding that was
previously held as `passenger_carry`. The implementation frees that known
zero and moves the already-live OWNED sign QReg into b[256]. This saves
one simultaneously live wire before result allocation. The old sign is
not copied, measured, re-encoded, or recomputed. The immediate field
negations use the same original physical sign wire as their control.
Consequently no sign is moved through the approximate multiplier.

## Ownership and Ghosts

`begin` moves the sign from Option into the multiplier vector after
freeing its old zero high bit. `finish` moves the same QReg back and
removes the tag from b. No stale high-bit ID is reacquired. The ownership
test occupies the former zero-bit ID with a live canary, then demands a
fresh canonical replacement while that canary is still live.

In the forward slope path, finish happens before any HMR of the
multiplier. Only its 256 meaningful coordinate bits are ghosted. The
omitted high bit was zero: measuring it would introduce no phase, and
resolving its ghost against the later canonical zero would also introduce
no phase for either outcome. Removing that random measurement can change
subsequent sampled measurement records, but not the corrected quantum
channel. The unchanged reconstruction allocates a 257-bit result; its
fresh zero high bit has no missing ghost obligation.

In the cancellation path, the tag stays in the unused slot through the
forward multiplier, both existing sign corrections, and the inverse
multiplier. Lambda ghosts are unchanged. After finish and temporary-result
cleanup, a fresh canonical zero high bit is allocated. The original
passenger-carry logic can then borrow this clean bit during reverse PZ,
or retain it as padding if that borrowing mode is disabled.

The live-set reduction is exactly one qubit during these payload windows.
The whole-circuit peak may remain limited by another phase, or by a
configurable exact scratch cap. Whole-circuit resource claims require
measurement after integration with other changes.

## Verification Protocol

All heavy work is remote-only under
`/root/ecdsafail-cpu/main/remote-run.sh`, using its single flock lock,
24 GiB virtual-memory cap, Rust 1.93.0, and three Cargo jobs. The actual
allocation is 16 vCPU, one RTX 5090, and 32 GB RAM. No nested locks,
local heavy fallback, or concurrent GPU nonce scanning is used.

The opt-in `MIDQ_PAYLOAD_SIGN_LOAN_SELFTEST=1` hook invokes the original
Simulator, with pre-reset checks and forced-zero, forced-one, alternating,
and deterministic-random measurement streams. Component coverage includes
nonzero high tags, the actual multiplier and its inverse, unchanged
coefficient donors, actual denominator-based reconstruction, both ghost
paths, and owned-handle transfer. Sampled field values are deterministic
full-width nonzero residues. The structural no-read assertion is not
restricted to those samples and does not assert that RFOLD arithmetic
is correct on every possible field input.

Full checks additionally require default-off byte identity against the
parity-only artifact, clean frozen baseline 9024 inputs, and matching
independent-corpus failure sets with no new phase-only or ancilla failures.
## Completed Remote Receipt

Component checks passed:

- 64 ownership cases, including occupying the former padding ID while
  allocating a fresh canonical high bit, and one fewer active qubit.
- 512 full-width tagged multiplier cases, covering tag 0 and tag 1,
  forward and inverse operation streams, donor preservation, phase,
  pre-reset checks, and the structural no-read assertion.
- 512 cases for each of the four combinations of loan off/on and
  forward-ghost/cancel-ghost, using actual multiplication and actual
  denominator-based numerator reconstruction. All data and phase checks
  passed, including the omitted zero-high ghost in the loaned variant.

Exact candidate flags, in addition to the unchanged 72190cc defaults:

    MIDQ_PACK_PZ_PARITY=1
    MIDQ_PAYLOAD_SIGN_LOAN=1
    MIDQ_PREFIX_QCAP=1018
    MIDQ_CHUNK_COMPARE_QCAP=1018
    MIDQ_PZ_VENT_QCAP=1018
    MIDQ_OUTER_VENT_QCAP=1018

On the original verified baseline's frozen 9024 inputs:

- Q: 1018, unchanged from the parity-only run at the same scratch caps.
- Average executed T: 14755891.239, down 750.080 from parity-only.
- Total executed T: 133157162538 over 9024 inputs.
- Average executed Clifford count: 37970420.559.
- Emitted operations: 82324144.
- Classical / phase / ancilla failures: 0 / 0 / 0.

Independent 9024 inputs retain exactly the baseline's five classical
failure IDs: 4174, 4968, 5407, 7617, 7854. There are zero ancilla-failing
batches. The three phase-failing batches flag inputs 4174, 5407, and
7617, all within that unchanged classical failure set. No new failing
input is introduced on either checked corpus.

With the new loan disabled, generation was byte-identical to the
parity-only artifact (SHA-256 beginning `2d8c5ff8cb2ac390`). The enabled
candidate's SHA-256 is:

    915af37233a6939410adf432802ed6777a224fe702010e53937c1eeba9c1bb64

All logs and preserved ops are in
`/root/ecdsafail-cpu/signs/validation/payload-sign-loan-20260905T195954Z-13821`.
The final gate is `completed.txt` (PASS). The global peak must be measured
again with the main branch's other memory changes and lower scratch caps;
these standalone results do not claim Q1009/T12.5M.
These are fixed-input regressions, not a new clean Fiat-Shamir nonce.
No defaults, generic arithmetic, or trusted harness files were changed.
No submission was made, and all heavy checks ran under the remote lock.
