# Joint hybrid-tail metadata codec

Parent: `d94e001`. This experiment is separate from that commit's recorded
Q1015/14.851M corpus validation. It also corrects the historical narrowing's
unproven canonical-endpoint assumption; see the audit correction in
`2026-09-05-narrow-coefficients.md`.

## Representation and Domain

Write q for the five-bit quotient tag, k for the seven-bit combined CTZ count,
s for the CTZ selector, and t for the counter/tape terminal flag. The inherited
metadata invariants are q in 0..18, k in 0..85, s in {0,1}, and
t=1 implies (q,k,s)=(18,0,0). This last implication uses the existing terminal
map: q is zero before quotient compression, and the mapped value pair is (1,1).
It is the already-used counter/tape terminal invariant, not a sampled range.

The rank is z=q+38*k+19*s+3250*t. For t=0, this equals
19*(2*k+s)+q and maps all 19*86*2 combinations bijectively to 0..3267.
The terminal tuple maps to 3268. Thus 3269 supported states fit in 12 bits,
instead of the 14 separate bits. CTZ=85, including the zero sentinel, is retained.
The extra nonterminal k=0,s=1 states are also retained. No empirical cutoff is
introduced. The codec is not claimed as a 14-to-12-bit compression of arbitrary
unconstrained 14-bit data.

For decoding, reversible division by 19 produces an eight-bit quotient and a
five-bit remainder. Ordinarily these supply (2*k+s) and q. For z=3268 the
quotient is 172 and remainder zero; XOR corrections k^=86 and q^=18 give the
terminal tuple, while s stays zero. The t flag is exactly [z=3268].

## Circuit and Ownership

Packing computes the rank in 12 temporary qubits, XOR-decodes it into the
original fields to clear them, then swaps the encoded word into the original
qtag-five + CTZ-seven physical slots. The temporary word and cleared selector
and terminal lanes are released. Unpacking performs the inverse operation and
returns the original qtag/CTZ slot IDs, with freshly owned selector/terminal
lanes. No freed ID is reacquired by assumption.

The division oracle copies the 12-bit rank to a remainder, uses eight exact
shifted comparisons/subtractions to obtain quotient bits, XORs decoded outputs,
and reverses those steps to clear the remainder and quotient. Terminal detection
runs only after this division scratch has been released. All constant operations
use full 12-bit arithmetic. The forward/inverse rank additions commute because
they have unchanged controls and a distinct destination.

The driver packs only after forward round 7 and unpacks immediately before
reverse round 7. Consequently the terminal flag remains available for every
decode of the first eight shared counter/tape slots. The encoded state remains
live through all later coefficient cells, checkpoint operations, and the payload
multiply. `Option<Packed>` explicitly records this state; `is_some()` on the
temporarily absent raw terminal flag is not used as the sole indication of
whether counter wires are shared.

## Measured Component Cost

The first remote component run exhaustively passed all 3269 supported states
under four measurement streams, including forced-zero, forced-one, alternating,
and pseudorandom measurement outcomes. It checks values, phase, every reset
before execution, scratch, and a deliberately intervening reuse of freed IDs.

With 942 unrelated retained qubits, total live storage changes
956 -> 954 -> 956. The measured pack and unpack peak is 996. This is 40 temporary
qubits above the raw starting state and 42 above the packed starting state,
slightly above the initial under-40 hypothesis but still below 1009 at this
modeled boundary. The codec costs 1445 emitted Toffolis to pack and 1416 to
unpack, before any savings from reusing the freed headroom. Two inversion
forward/reverse pairs would add 5722 emitted codec Toffolis in total.

The first integrated 224-round tail checks passed all 192 unshared and 960
shared-counter cases, both with and without checkpointing. The subsequent locked
run with the conservative wide-endpoint correction also passed all four sets.
Its endpoint test covers 294 values including u>=p, both sign controls, four
measurement streams, and both entry layouts. It verifies the entire 257-bit
p-u word, a high-bit-sensitive payload/phase observable, inverse restoration,
scratch, and every reset before execution. The independent 2824-case cell
layout regression also passes. These tests do not assume u<=p at the endpoint.
No full-circuit generation, new nonce search, baseline-corpus repeat, or submission
was performed for this metadata experiment. No full-circuit Q/T claim is made.

## Flags and Receipts

New option: `MIDQ_TAIL_METADATA_CODEC=1`. It activates only when quotient
compression and shared-counter metadata are present; otherwise raw metadata is
retained. Use alongside the existing narrow/park flags after applying the
wide-endpoint correction. The tests retain the original 360 PZ steps, 224 tail
rounds, and existing arithmetic bounds.

Remote directory: `/root/ecdsafail-cpu/coeff`.
Initial component receipts: `validation/tail-metadata/`.
Corrected endpoint/metadata receipts: `validation/tail-metadata-wide-endpoint/`.
All heavy work goes through the coordinator's global `remote-run.sh` lock with
`CARGO_BUILD_JOBS=3`; no local heavy jobs are permitted.
