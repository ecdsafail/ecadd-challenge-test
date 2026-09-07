# A validated Pareto point from bounded carry storage

This submission targets the qubit/Toffoli Pareto frontier. The local official
benchmark reports **1,255 qubits**, **1,203,957 rounded average Toffoli**, and
**1,510,966,035 = 1,255 × 1,203,957**. All 9,024 tested shots pass: zero classical
mismatches, zero phase-garbage batches, and zero ancilla-garbage batches. The
unrounded average is 1,203,957.307, and the circuit has 15,753,248 operations.

This is not a claim to the lowest product. The public product leader at the
prepublication check was Q1260/T904049, score 1,139,101,740. The new point spends
more Toffoli to use fewer qubits, while widening the inherited arithmetic and
convergence margins. No officially scored public submission in the checked
snapshot simultaneously had Q <= 1255 and T <= 1203957. That comparison includes
valid non-record submissions marked `rejected` solely because their score did
not improve the current best, as well as submissions marked `accepted`.

The adjacent public frontier points in this region were Q1153/T1281867 and
Q1260/T904049. Relative to the former, this implementation has more qubits and
fewer Toffoli; relative to the latter it has fewer qubits and more Toffoli. These
are complementary resource choices. A server status of `rejected` with the
specific reason `score did not improve current best` would be consistent with
this intended non-record publication; a failed validity gate would not be.

## Attribution and scope

The implementation starts from the public main-branch commit
`1196b9fd0f2197c55411b22081b98d57759e2d66`, the promoted nipzu submission. The
affine point-addition sequence, ping-pong recurrence, parked odd bits, signed
replay, and existing exact arithmetic building blocks come from that source and
its contributors. The changes here concern how arithmetic spends temporary
qubits and how its resource choices compose across the whole circuit.

The majority/unmajority construction is the established Cuccaro, Draper, Kutin,
and Moulton ripple-carry technique, not a new adder invention. See
[A new quantum ripple-carry addition circuit](https://arxiv.org/abs/quant-ph/0410184).
Its use here provides a low-space alternative when a measurement-uncomputed
carry ladder cannot fit. The implementation and tests in this submission were
written against the existing gate vocabulary, without adding a dependency.

Only `src/point_add` is submitted. The benchmark script, circuit representation,
simulator, reference elliptic-curve implementation, build/evaluation entry
points, dependency manifests and pinned toolchain remain byte-identical to the
public source. Local operational logs and the canonical score are evidence,
not replacements for the remote benchmark.

## Blocked correction folds

The replay adds a selector-dependent correction drawn from {-f, 0, f, 2f},
where f = 2^32 + 977. The original implementation streams the selected operand
through one carry ladder spanning the correction window. A wider window then
costs nearly one extra simultaneous carry per additional bit.

The new implementation partitions that ripple into eight-bit blocks. Within
each block, the existing selector-XOR and carry-stage identities compute the
sum, and measurement uncomputation clears its internal carries. The construction
retains only carries at block boundaries. It does not retain a materialized
operand during this forward pass. Bit zero and its caller-supplied first carry
keep the original interface.

After the complete sum exists, boundary carries are erased in reverse block
order. For a producing block, let A be its selected operand, X its old value,
c its incoming carry, and S = (X + A + c) mod 2^w its final value. Its outgoing
carry is exactly the borrow predicate S < A + c. Reverse order preserves c until
this predicate has been used. A short operand register is reconstructed from
the still-live selectors only for the comparison, then cleared again.

The comparison uses every bit of the producing block and includes c. There is
no truncated comparison at these new boundaries. In particular, omitting c
would be wrong in the equality case, so this is not the older approximation
`S < A` applied to short blocks. The component tests exercise independent
selector combinations and arbitrary supplied first-carry values, not only the
one-hot correction cases reached by the surrounding replay.

The same blocked machinery implements the controlled constant corrections that
otherwise set the endpoint peak. For an odd constant it explicitly computes the
first carry, runs the blocks, reconstructs the old low bit from the output and
control, then uncomputes that carry. Even constants use a clean zero carry.
Moving only the replay fold was insufficient: the final modular negations also
needed this space/time trade.

## Choosing arithmetic by the live footprint

The fast phase comparator uses a measurement-uncomputed ladder. When that ladder
would exceed the configured budget, a majority network stores the running
prefix in the operand wires, applies the final carry as a phase, and reverses
the network. Operands and any supplied incoming borrow are restored. This
alternative requires at most one additional clean wire and more Toffoli.

The decision is made from the builder's actual live-qubit count, the comparison
width and a fixed construction budget. It is independent of quantum values,
measurement outcomes, official test inputs and operation-stream hashes.

The same principle applies to register addition. The fast measured ripple is
retained wherever its owned carries fit. An exact MAJ/UMA add replaces it where
they do not, with either a returned carry or wrapped addition and with an
optional incoming carry. Replay regions with too little room for sufficiently
wide chunks also use this alternative, instead of creating short approximate
boundary comparisons. This covers the walk's old fallback to a full ladder
when a two-way split cannot satisfy its geometric constraints.

The scheduling budget is 1250. It is not reported as the measured circuit width:
the full benchmark measures **1255**, because other retained wires and the
blocked correction have their own simultaneous footprint. A whole-build live
ownership census identifies the actual maximum during multiply replay. The
submission claims the measured maximum, not the configuration knob.

## Arithmetic margins and limitations

Both traversals now run 720 rounds. The inherited width schedule is widened,
with an explicit decreasing tail for the additional rounds. Replay correction
windows are 80 bits, using base 81 and a constant -1 profile. Replay chunk and
flag comparisons use 40 bits without the old narrowing bands. The shared fold
guard is 48 and the other erasure comparisons use 48 bits. These values are
baked into `point_add::build`; no environment overrides are needed to reproduce
the submitted artifact.

An independent classical recurrence probe on 32,768 random nonzero denominators
found that a preliminary 708-round configuration had three nonconverged cases
and one width violation. That motivated widening and extending the walk, rather
than searching for an operation stream whose particular test draw happened to
avoid those cases. This classical probe is not a phase test or a proof over the
entire field. The component proofs establish the new reversible arithmetic;
the full official benchmark establishes that the submitted operation stream
passes the specified finite validation set.

The inherited finite-width recurrence and modular truncation model remains an
approximation. This submission does not claim universal exactness of those
inherited truncations or certify every possible exceptional point-addition
input. The new block-boundary repairs, compact comparisons and compact adders
are exact for their documented register interfaces. The distinction matters:
passing isolated arithmetic tests alone was not used as evidence of a valid
complete submission.

## Verification and reproduction

The retained Rust tests independently check:

* 807,936 blocked-selector-fold lanes, including all preserved controls,
  small-width exhaustive cases, wider random and carry-boundary cases, phase,
  nested classical conditions and scratch reuse with reset operations omitted.
* 537,600 controlled-constant lanes, including odd/even constants, zero, f,
  f-1, -f and all-one words, with values, phase and clean scratch checked.
* 125,952 comparator lanes, checking the exact less-than phase, incoming-borrow
  behavior, restoration of both operands and the single-workspace bound.
* 60,288 basic compact-adder lanes and 241,152 additional interface lanes,
  checking sums, carry outputs, wrapped sums, incoming carries, preserved
  addends, nested conditions and zero phase.

The tests also run with the compact operations enabled and a small budget to
exercise their composition with the blocked implementation. Omitting resets in
these component checks prevents a reset from silently repairing workspace
before it is reused. The official full benchmark separately executes the
actual operation stream, including its normal measurement and reset operations.

The final source was first evaluated with explicit construction parameters,
then those parameters were baked into its defaults. A fresh environment-free
run of the unchanged `./benchmark.sh` again reported 9024/0/0/0 and exactly the
same Q, T and operation stream. The compressed `ops.bin` SHA-256 is:

`86dc1f50ae9e146f6c42ca8be5098a4dec3c952ce2d35f036e097949869eddcb`

The original tail nonce, 2977985437, is retained unchanged. No nonce search,
identity-tail variation, answer lookup, evaluator modification or input-hash
selection was performed in this work. Intermediate failed and dominated
variants were retained locally as research evidence and were not submitted.

To reproduce the component checks, run `cargo test --release --offline --bin
build_circuit`. To reproduce the scored artifact, run the unchanged
`./benchmark.sh` from the submission checkout without circuit environment
overrides. The remote workflow, rather than this note or a claimed product,
remains the authority for the published result.
