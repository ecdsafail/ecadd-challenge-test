# Bounded Parity Storage and Rejected Input-Sign Motion

Base: 72190cc, the independently verified Q1019 circuit. The new feature
is enabled only by `MIDQ_PACK_PZ_PARITY=1`; its default is off.
All value/coefficient/quotient widths, 360 PZ steps, 224 tail rounds,
approximation guards, and existing normalization routines are unchanged.

## Exact One-Qubit Parity Encoding

At handoff the physical raw `cb` has at most 248 bits. Quotient flushing,
quotient compression, and the terminal/counter map leave `cb` unchanged.
Write its unsigned value as u, with 0 <= u < 2^248, and PZ parity as pi.
The existing sign normalization transforms this field into

    cb = u          if pi = 1,
    cb = p - u      if pi = 0.

This is the existing exact 257-bit controlled p-minus-u permutation, not
an approximate modular fold. With p=2^256-2^32-977, the former value has
bit 255 clear and the latter has bit 255 set. This includes u=0: its
negated representation is p, not a canonicalizing rewrite to zero.
Thus cb[255] = 1 XOR pi. X(pi) followed by CX(cb[255],pi) clears pi
exactly, allowing its owned QReg to be released. No measurement is added.

The forward code releases parity BEFORE power-of-two normalization. The
reverse first undoes that normalization, then allocates a FRESH parity
wire and computes 1 XOR cb[255]. It reverses the original sign operations
and proceeds through the unchanged PZ schedule. A freed handle is never
retained or manually reacquired. Option::take removes its owner; the
fresh allocation can use any available physical ID. Tests deliberately
occupy the former ID until restoration to verify this property.

The codec declines borrowed parity handles and source cb widths above
248. This is a compile-time source-width guard, not a sampled-data guard.
The original tail helpers retain their borrowed-wire interface for their
existing component tests, so those wider synthetic cases keep parity.

On a terminal state the counter codec maps (A,B,ca,cb)=(0,1,p,u) to
(1,1,p-u,u). Sign normalization makes the two coefficient fields equal,
either u or p-u. The parity identity still holds. The value pair stays
(1,1), all logical tape signs remain zero, and every one of the 256
physical counter encodings is preserved by the unchanged tape codec.
The last-four-round checkpoint is unchanged and retains the same
endpoint selectors. Parity reconstruction happens only after its
inverse and the complete value/coefficient replay.

## Why Input-Sign Reclamation Was Declined

Odd two's-complement negation itself is cheap and exact:
N_w(z)=z XOR (2^w-2). It is implemented by CNOT on all bits above bit zero.
Negating both rows flips both second-lowest bits, so their XOR decision
bit is unchanged. The selected sum/difference is always 2 modulo 4.
For w>=3 it therefore cannot be the exceptional signed minimum; the
wrapped arithmetic right shift commutes with common negation. During
signed shrinking, each discarded bit is XORed with the retained sign,
so common negation also preserves its zero/reset-error predicate.

However, both production handoff rows occupy 85 bits, equal to the
initial signed-tail width. Their original MSB is not structurally zero.
Trying to erase input `sgn` from the negated row MSB leaves that original
MSB behind. For example, the positive unsigned odd word 2^84+1 already
has its MSB set. We do not add a positivity assumption or widen/shrink
the inherited schedule to make this erasure appear valid.

There is a second independent obstruction to removing the three outer
sign corrections. Moving a sign correction through the existing
APPROXIMATE multiplier is not a value-exact circuit identity. Let
R=2^32+977, h=2^255+2^72-1, a=p-h, and multiply by 2 with input sign 1.
The real emitted circuit gives:

    baseline: N(M(a,2)) = 2^73 + R - 2,
    moved:    M(N(a),2) = R - 2.

The baseline multiplication does not overflow 256 bits. The moved
version doubles h to 2^256+2^73-2, then its fixed 73-bit +R fold loses
the carry into bit 73. The original result is correct; the rewrite
introduces an error. A component test executes both real multiplier
circuits and confirms the witness under four measurement streams.
Consequently the input sign, all three outer field negations, and the
multiplier inputs remain unchanged in this patch. No input-sign feature
flag is shipped.

## Component Verification

Build the untracked `count_tof` diagnostic on the authorized remote host,
then run `MIDQ_SIGN_STORAGE_SELFTEST=1 target/release/count_tof` there.
All heavy builds, generations, and evaluations must pass through
`/root/ecdsafail-cpu/main/remote-run.sh`, which holds the single global
`flock` lock. Do not nest locks or use a local fallback. The allocation
is 16 vCPU, one RTX 5090, and 32 GB RAM, regardless of host-level readings.
The signs work directory is `/root/ecdsafail-cpu/signs`; the wrapper pins
Rust 1.93.0 and CARGO_BUILD_JOBS=3. No GPU nonce job accompanies validation.
This hook runs tests and exits before a point-addition circuit is emitted.
It is absent from the normal build path when the variable is unset.

- 174720 signed odd-pair/input-sign/value-update cases over widths 3..8:
  exact transcript, wrapped forward value, inverse, phase, pre-reset.
- 2000 full-width parity cases: zero, powers of two, maximum 248-bit
  value, dirty donors, both parities, and forced fresh ownership.
- 4352 complete-tail cases per checkpoint mode (off and on), covering
  both parities, odd/even handoff, every terminal counter, and a live
  sign spectator; all data restored with clean scratch and phase.
- The rejected multiplier-sign-motion witness runs the actual arithmetic.

The simulator is the original crate Simulator. Every reset in accepted
component circuits is checked BEFORE it executes. Measurement streams
are forced zero, forced one, alternating bits, and deterministic random;
these are not a claim to enumerate every exponentially many global
measurement assignment. The codec itself adds only X/CX and clean
allocation/reset, so its branch-independent correctness follows from
the invariant above, not measurement sampling.

## Remote Validation Receipt

All checks completed under Rust 1.93.0 on the authorized remote host. The
initial frozen evaluation was stopped for the RAM-allocation correction;
its restart and subsequent checks used the global locked wrapper. No
build or generation was repeated after its successful receipt existed.

Exact candidate flags, in addition to the unchanged 72190cc defaults:

    MIDQ_PACK_PZ_PARITY=1
    MIDQ_PREFIX_QCAP=1018
    MIDQ_CHUNK_COMPARE_QCAP=1018
    MIDQ_PZ_VENT_QCAP=1018
    MIDQ_OUTER_VENT_QCAP=1018

- Q: 1018 (baseline 1019).
- Average executed T on the baseline's frozen 9024 inputs: 14756641.319.
- Total executed T: 133163931260 across 9024 inputs.
- Frozen classical / phase / ancilla failures: 0 / 0 / 0.
- Emitted operations: 82321089.
- Independent 9024: the same five classical failing inputs as baseline,
  exactly 4174, 4968, 5407, 7617, 7854. Both have zero ancilla failures.
  Both have three phase-failing batches; their phase-failing inputs are
  contained in the existing classical failure set, with no new failures.

With all scratch caps reduced to 1018, the standalone average T is
7909.806 above the baseline's 14748731.513 on the same frozen inputs.
This is a one-qubit tradeoff, not a standalone simultaneous Q/T reduction
or an attainment of the main Q1009/T12.5M target. The parity codec itself
adds no Toffoli gates. Its interaction with allocation-sensitive exact
arithmetic must be measured again when combined with other optimizations.

The default-off generated file was byte-identical to the baseline.
SHA-256 values:

    baseline: 403563ee39701de8af489f680fc4efd0f26afd97dcc4ab4db738385aae74908f
    candidate: 2d8c5ff8cb2ac3903e9ca51b40174a85ffd7c8f5e053665b7e850a26ffb4d1e3

Complete remote logs and immutable candidate ops are in
`/root/ecdsafail-cpu/signs/validation/parity1018-20260905T185111Z-8461`.
The final gate is `completed.txt` (PASS); frozen metrics are in
`frozen9024-resumed.log`, and the paired independent records are in
`independent-baseline.log` and `independent-candidate.log`.
This is a fixed-input regression receipt, not a new clean Fiat-Shamir
nonce. No submission was made, and no trusted harness file was changed.
