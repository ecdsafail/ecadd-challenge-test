# Narrow coefficient storage and parked endpoint selectors

Parent source: `72190cc` (independently verified Q1019 circuit).
Branch: `codex/midq-coeff-1009`.

## Audit Correction

The resource/corpus measurements below describe historical commit `d94e001`.
They are not proof that its 256-bit endpoint negation preserves the full
baseline channel. In particular, for `u=p+1` and an enabled negation, the old
257-bit result is `2^257-1`, while the narrowed result omits the meaningful high
bit. There is no established proof that all baseline-successful inputs keep
endpoint coefficients at most p. Do not integrate that endpoint shortcut.

The corrected implementation keeps incoming high bits through the first two
forward tail cells, restores both rows to 257 bits before endpoint negations,
and retains the complete 257-bit ca through the payload and reverse endpoint.
It narrows again only after both rows have actually passed the existing HMR
carry cleanup: after the four reverse checkpoint cells, or after the first two
ordinary reverse cells when checkpointing is disabled. Narrowing is disabled
when measured comparison cleanup is disabled. These release points use an
explicit measurement/reset fact, not a canonical-range assumption.

The new endpoint edge test includes u>=p and checks the full 257-bit result,
a payload-sensitive high-bit/phase observable, reverse restoration, and all
scratch. See the joint metadata note for the new component-only results; the
historical Q/T table below must not be assigned to the corrected combination.

## Changes

The two hybrid-tail coefficient rows retain 256 data lanes between arithmetic
operations. The active signed addition owns a separate, initially zero overflow
lane; it applies the original Cuccaro sum, full-width pseudo-Mersenne fold, and
comparison cleanup, then releases that lane. The source never needs a persistent
overflow lane. Modular halve/double formulas are unchanged.

The historical patch relied on a canonical coefficient contract at release and
endpoint boundaries. The correction above removes that additional assumption.
No PZ/quotient/value-width envelope, CTZ support, iteration count, modular
correction window, or comparison guard is narrowed.

The historical multiplication interface accepts a 256-bit coefficient source while retaining
the multiplier's complete 256-bit loop. In particular, the loop length is taken
from the still-257-bit multiplier, not the narrowed source. A zero source-padding
lane is allocated only within each integer-addition step and returned before the
fold and phase cleanup. With no vent headroom, the padded source uses the exact
zero-vent hybrid adder instead of the more expensive explicit-overflow controlled
Cuccaro fallback. The original multiplication folds and cleanup conditions are
otherwise unchanged. The corrected driver deliberately passes the complete
257-bit endpoint ca instead, so any nonzero high bit is preserved; do not count
the historical payload-source width saving for the corrected driver.

For the four-round endpoint checkpoint, the two endpoint signs are functions of
the retained six checkpoint bits. `park_selectors` repeats their existing XOR
lookups to clear them, then releases their owned QRegs across the payload
multiplication. `restore_selectors` allocates fresh zero lanes and repeats the
same lookups before reverse endpoint correction. Both lookups read only the six
retained bits; neither reads the other selector. No checkpoint information is
discarded, and no particular freed physical ID is assumed when restoring them.

The temporary overflow and source-padding lanes are fully released before the
existing loaned odd low bits are reclaimed. There is no added register alias or
new simultaneous owner of those physical lanes.

## Options

The patch is opt-in. The validated configuration adds:

```sh
MIDQ_NARROW_COEFFICIENTS=1
MIDQ_PARK_CHECKPOINT_SELECTORS=1
MIDQ_OUTER_VENT_QCAP=1009
MIDQ_PZ_VENT_QCAP=1009
MIDQ_PREFIX_QCAP=1009
MIDQ_CHUNK_COMPARE_QCAP=1009
MIDQ_CHUNKED_PREFIX=1
MIDQ_CHUNK_COMPARE=1
```

Other route defaults, including PZ cut 360, tail length 224, and nonce 16445,
remain those of `72190cc`. No nonce search or submission was performed. Reducing
the scratch caps is an exact recomputation/vent trade, not an arithmetic bound.

## Component Evidence

- 2,824 edge/random coefficient inputs, three operations, four measurement
  streams, and both 256/257-bit layouts. Outputs agree at both forward/reverse
  checkpoints; sources and sign controls are preserved; scratch and every reset
  are checked. The baseline itself has phase-sensitive arithmetic edge inputs
  (364 and 316 positions in the two cell tests). These are recorded as inherited
  support, not silently counted as clean all-input field arithmetic. The narrow
  layout matches forced-measurement phases and adds no observed phase-failure
  support in the tested streams.
- Complete 224-round tail, normalization, quotient codec, endpoint, and reverse:
  192 cases without shared counter storage and 960 with it, for each of the
  checkpoint-disabled and checkpoint-enabled configurations. Value, phase,
  scratch, pre-reset cleanup, and restored register ownership checks pass.
- Existing stationary-coefficient, counter-wrap, and prefix-width obligation
  checks also pass. The trusted evaluator, simulator, field reference, dependency
  files, and benchmark script were not modified.

## Remote Circuit Evidence

Heavy work ran only in `/root/ecdsafail-cpu/coeff` after laptop offload. Remaining
checks use the coordinator's global `remote-run.sh` lock, with one heavy job at a
time and `CARGO_BUILD_JOBS=3`. The allocation is 16 vCPUs and 32 GB RAM.

The frozen corpus is derived from the verified parent artifact at
`/root/ecdsafail-cpu/baseline/ops.bin`, not from this candidate's changed hash.
All 9,024 inputs pass with zero classical, phase, and ancilla errors:

| Metric | Candidate |
|---|---:|
| Peak logical qubits | 1,015 |
| Average executed Toffolis | 14,850,950.624 |
| Total executed Toffolis | 134,014,978,435 |
| Emitted operations | 82,069,380 |

This is a memory improvement, not a standalone Toffoli improvement over the
Q1019/14.749M verified configuration. The smaller scratch caps consume some gate
budget. Main-worker arithmetic/cleanup changes are not included in this isolated
measurement and must be measured again after integration.

Paired independent 9,024-input comparison passed after the locked remote resume.
Both circuits have the same five classical and any-channel failure IDs:
`4174, 4968, 5407, 7617, 7854`. Neither has ancilla failures. Candidate phase
failures were `4174, 4968, 5407, 7854`, versus `4174, 4968, 5407` for the parent;
they remain within the same classical-failure support. Individual phase outcomes
need not match after changing the measurement stream. This finite corpus is
regression evidence, not a universal correctness proof or a clean nonce for the
candidate.

Candidate `ops.bin` SHA-256:
`d9e91c938bf3f3b6b9012735e8c1f99a37e40a7b7fbee1b795fd2f7be4f999d2`.
Frozen parent `ops.bin` SHA-256:
`403563ee39701de8af489f680fc4efd0f26afd97dcc4ab4db738385aae74908f`.

Full receipts remain on the remote machine in
`/root/ecdsafail-cpu/coeff/validation/narrow-coefficients/`:
`build.log`, `components.log`, `full-tail.log`, `generation.log`,
`frozen-9024.log`, `independent-candidate.log`, `independent-baseline.log`,
`paired-summary.txt`, and `ops-sha256.txt`. The interrupted independent log was
preserved separately; completed circuit and frozen receipts were not replaced.

## Integration

Enable both new flags when integrating this optional patch. The rfold changes
only concern the 256-bit multiplication-source interface and integer-adder
padding; they do not implement the main worker's revised outer phase cleanup.
That helper must continue to accept either 256- or 257-bit sources without
assuming a source overflow lane exists.

The optional quotient-tag/CTZ radix encoding was not implemented. A possible
exact future encoding is `z = 86*qtag + k` for `qtag` in 0..18 and CTZ `k` in
0..85, including the existing zero sentinel. This needs 11 bits, with the
separate selector making 12 instead of 13 total metadata bits. Any realization
still needs a fully costed reversible pack/unpack and cleanup schedule.
