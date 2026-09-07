use std::str::FromStr;

use alloy_primitives::U256;

use crate::circuit::{BitId, Op, QubitId};
use builder::Builder;
use classical::{coord_add3x, coord_rsub, coord_sub};
use pingpong::{divide, multiply};
use square::sub_square;

mod builder;
mod classical;
mod compare;
mod const_arith;
mod modular;
mod pingpong;
mod record;
mod square;

const N: usize = 256;

const SECP256K1_P: U256 = U256::from_limbs([
    0xFFFF_FFFE_FFFF_FC2F,
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
]);

/// Pin a tuning knob, unless the environment already overrides it.
///
/// **The single-line call shape is load-bearing.** `src/bin/grind` `include_str!`s
/// this file and text-searches it for the literal needle
/// `set_default_env("NAME", ` to recover a pinned value, because the grinder's
/// model has to track the circuit's tuning and must not keep its own copy.
/// Seven knobs are read that way today: `PP_ROUNDS_MUL`, `FOLD_GUARD`, both
/// `PP_REPLAY_FOLD_WINDOW*`, `PP_FOLD_WIDEN`, `PP_FOLD_PROFILE` and
/// `PP_WIDTH_SCHEDULE` -- the last two as string literals, the schedule as a
/// `concat!` of them. So a pin's value must stay a literal right here in the
/// call: routing it through a constant, a table or a helper compiles fine and
/// makes the grinder panic at startup -- or, worse, screen against tuning the
/// circuit no longer has. (It used to route the schedule and the band tables
/// through `pub const DEFAULT_*` in `pingpong.rs`, which is exactly the shape
/// that needed a second scraper and a second file.)
///
/// The `unsafe` is required from edition 2024 on, where `set_var` is an
/// `unsafe fn`; this crate is on 2021, where it is merely allowed.
fn set_default_env(name: &str, value: &str) {
    if env_raw(name).is_none() {
        unsafe {
            std::env::set_var(name, value);
        }
    }
}

/// Defines a memoised `usize` accessor for a tuning knob that [`build`] pins.
///
/// The knob is read once through [`required_env`] and cached, so every consumer
/// sees one value and the environment is not walked per call site. That reader
/// deliberately has no fallback, so a dropped pin panics rather than silently
/// emitting a different circuit.
///
/// Every knob that reaches a construction as a width in bits goes through here;
/// ten do. The five that do not are the ones that are not a width: the tail
/// nonce, read in [`build`], and the four strings, each read once behind its own
/// `OnceLock` in `pingpong` -- the width schedule, and the three `width:offset`
/// band tables (`PP_FOLD_PROFILE`, `PP_CHUNK_SHAPE`,
/// `PP_FLAG_SHAPE`).
macro_rules! pinned_env {
    ($name:ident, $env:literal) => {
        fn $name() -> usize {
            static SLOT: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
            *SLOT.get_or_init(|| $crate::point_add::required_env($env))
        }
    };
}
// The submodules are declared above this macro, so textual scoping does not
// reach them; this re-export is what makes `crate::point_add::pinned_env` a path
// they can import. No visibility marker: like everything else in this file it is
// private to `point_add` and therefore already visible to its descendants, which
// is all the submodules need. (`pub` would be E0364 anyway -- a `macro_rules!`
// without `#[macro_export]` cannot be re-exported wider than the crate.)
use pinned_env;

/// The raw value of `name`, or `None` when it is unset, empty, or not UTF-8.
///
/// **Every environment read in this tree goes through here**, so the rule can be
/// stated once: empty counts as unset. That makes `KNOB= ./build_circuit` clear
/// an override rather than pin the empty string, and it is the same rule
/// `src/bin/grind` applies when deciding whether an override beats a pinned
/// literal -- the two have to agree, or the grinder models a different circuit
/// than the one that was built.
///
/// `var` rather than `var_os`: a knob that is not UTF-8 is not a knob, and
/// reading it as absent puts it on the pinned default instead of carrying a
/// value nothing downstream can parse.
fn env_raw(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// A diagnostic switch. Off when unset, empty, or set to a value that plainly
/// means no -- `0`, `false`, `no`, `off`, in any case. Anything else is on, so
/// `KNOB=1` and `KNOB=please` both enable and only a deliberate `KNOB=0`
/// disables.
fn env_flag(name: &str) -> bool {
    env_raw(name).is_some_and(|value| {
        !matches!(
            value.to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        )
    })
}

fn required_env<T: FromStr>(name: &str) -> T
where
    <T as FromStr>::Err: std::fmt::Display,
{
    let Some(raw) = env_raw(name) else {
        panic!("{name} is not set; every tuning knob must be pinned in point_add::build")
    };
    raw.parse().unwrap_or_else(|e| {
        panic!(
            "{name}={raw:?} is not a valid {}: {e}",
            std::any::type_name::<T>()
        )
    })
}

/// A knob that may be absent: `None` when unset, empty or unparseable.
///
/// The counterpart to [`required_env`], and the split is the point. Anything the
/// *circuit* reads goes through `required_env` and panics on a missing pin,
/// because quietly building a different circuit is the one failure nothing
/// downstream can catch. Anything that merely *observes* the build -- the
/// recorders in [`record`] -- have the opposite duty and must default to off, so
/// it comes through here.
fn optional_env<T: FromStr>(name: &str) -> Option<T> {
    env_raw(name)?.parse().ok()
}

// ─── The truncation width every constant fold shares ───────────────────────
//
// Every approximation outside the ping-pong replay is one of exactly two
// shapes, and the balance rule says both want ONE value each, not one per
// caller. Equalising `d(lambda)/d(Toffoli)` across sites gives `err / u` equal,
// where `u` is 1 for an unconditional site and 1/2 for a compare (whose Toffoli
// sit under a `push_condition` and execute half the time). So:
//
//   * every constant-fold site wants the same per-call error, and
//   * every measured-erasure compare wants HALF that error, i.e. one bit more.
//
// Notably that does not depend on the call count: a 29-call site and a 2-call
// site want the same width. The replay's own windows are not here because they
// are pinned by peak geometry rather than by error.
//
// Only the first of the pair is declared here, because only the first is shared
// -- `fold_guard` reaches `modular`, `square` and `pingpong`. The compare width
// it is derived against lives with its single caller, as
// `modular::erase_compare`; both are still pinned together in [`build`], which
// is where the two values have to be read side by side to stay one bit apart.
pinned_env!(fold_guard, "FOLD_GUARD");

/// The candidate itself: `(x, y) += (ox, oy)` on secp256k1 in affine
/// coordinates, with `(ox, oy)` classical.
///
/// The chord-and-tangent formulae in eight phases, each named for the `set_phase`
/// report `build_circuit` prints. `x`/`y` are the quantum coordinates and are
/// overwritten in place; every scratch qubit each phase takes is returned to |0>
/// before the next one starts.
fn build_point_add() -> Vec<Op> {
    let circ = &mut Builder::new();
    let x: &[QubitId] = &circ.alloc_qubits(N);
    let y: &[QubitId] = &circ.alloc_qubits(N);
    let ox: &[BitId] = &circ.alloc_bits(N);
    let oy: &[BitId] = &circ.alloc_bits(N);

    circ.set_phase("coord_x_sub"); // x2 -= ox
    coord_sub(circ, x, ox);

    circ.set_phase("coord_y_sub"); // y2 -= oy
    coord_sub(circ, y, oy);

    circ.set_phase("divide"); // y2 /= x2
    divide(circ, y, x);

    circ.set_phase("coord_add3x"); // x2 += 3*ox
    coord_add3x(circ, x, ox);

    circ.set_phase("square"); // x2 -= y2^2
    sub_square(circ, x, y);

    circ.set_phase("multiply"); // y2 *= x2
    multiply(circ, y, x);

    circ.set_phase("coord_y_sub_final"); // y2 -= oy
    coord_sub(circ, y, oy);

    circ.set_phase("coord_rsub_final"); // x2 = ox - x2
    coord_rsub(circ, x, ox);

    circ.declare_qubit_register(x);
    circ.declare_qubit_register(y);
    circ.declare_bit_register(ox);
    circ.declare_bit_register(oy);
    circ.finalize_records();
    circ.take_ops()
}

/// Pin every tuning knob and build the circuit.
///
/// These pins are the circuit's entire configuration. Everything under
/// [`build_point_add`] reads them back through [`required_env`], which panics on
/// a missing one, so a dropped pin stops the build instead of quietly emitting a
/// different circuit. [`set_default_env`] does not overwrite, so a value already
/// in the environment wins -- that is how the knobs are swept.
///
/// # Why these values differ from the inherited ones
///
/// This configuration is chosen so the circuit is CORRECT for a random
/// Fiat-Shamir draw, not for one ground op stream. The inherited pins (696
/// rounds, a width schedule fitted to the typical magnitude rather than to an
/// envelope of it, and 20/21-bit measured-erasure compares) leave an expected
/// ~18 wrong shots per 9,024-shot run: they only pass because the tail nonce was
/// ground against their exact op stream. Measured directly -- the inherited
/// circuit was re-run under 24 arbitrary tail nonces and failed all 24, with 12
/// to 25 classical mismatches and 8 to 20 phase-garbage batches each.
///
/// There is no tail nonce here at all. The 96-op identity tail and the
/// `TAIL_NONCE` knob that steered it have been removed, so nothing in this tree
/// can select a favourable shot sample.
///
/// Every width below is set from a measured error budget. The total expected
/// number of wrong shots per 9,024-shot run is about 0.044, i.e. a per-input
/// error rate near 5e-6, and the dominant terms are the two walks' convergence
/// tails and the width schedule's margin. Measured: 31 of 31 independent
/// Fiat-Shamir draws pass with zero mismatches and zero phase garbage, which
/// bounds the rate at 0.093 with 95% confidence. See
/// research/claude-run/REPORT.md for the model and the measurements.
pub fn build() -> Vec<Op> {
    // ── The two walks ──────────────────────────────────────────────────────

    // Per-round walk register width, run-length encoded as `width x rounds`.
    // Its LENGTH is the divide walk's round count, so there is no separate
    // rounds knob on this side.
    //
    // GENERATED, and generated differently from the inherited schedule. Every
    // entry is an upper bound on the width the recurrence actually needs at that
    // round, taken from a direct measurement of the recurrence rather than from a
    // single fitted margin, and non-increasing so `shrink_to` can apply it.
    //
    // Why the shape and not just the level: the width the recurrence needs is
    // near-deterministic in the early rounds and has a long thin tail in the
    // late ones. A schedule that is a fixed number of bits below the worst case
    // is therefore too narrow at the head and too generous at the tail. Too
    // narrow at the head is not a rare miss: `shrink_to` frees a wire that is
    // not a sign copy, `Builder::free`'s reset leaks a random phase, and the run
    // fails. That is what the inherited configuration's phase-garbage batches
    // were.
    //
    // One trap worth writing down: make the schedule non-increasing with a
    // suffix maximum, never by clamping a round down to its predecessor. Round 1
    // is pinned to 257 by `round1_forward`'s assert while round 2 needs 258, so
    // the downward clamp silently puts round 2 below its requirement and fails
    // about 30% of shots.
    //
    // 730 rounds, not 696. The walk must reach |u| = |v| = 1 before `endpoint`
    // is valid, and 696 rounds leaves about 4 shots in every 9,024 short of that
    // on their own. 730 takes it to about 0.012. Do not truncate or pad this
    // literal; its length IS the divide depth, and a different depth needs a
    // schedule generated for that depth.
    set_default_env(
        "PP_WIDTH_SCHEDULE",
        concat!(
            "259,258x23,257x4,256x4,255x4,254x5,253x2,252x3,251x6,250x3,249x6,248x2,247x2,246x4,245x2,",
            "244x4,243x3,242x2,241x3,240x3,239x3,238x5,237x3,236x3,235x5,234,233x3,232x4,231x3,230x3,",
            "229x3,228x3,227x2,226x2,225x4,224x6,223x3,222x3,221x2,220x5,219x2,218x4,217,216x3,215x2,",
            "214x4,213x3,212x3,211x4,210x2,209x4,208x3,207,206x2,205x4,204x3,203x2,202x3,201x4,200x4,",
            "199x2,198x4,197x2,196x3,195x6,194x2,193,192,191x3,190,189x6,188x3,187,186x4,185x3,184,",
            "183x3,182x3,181x2,180x3,179x3,178x3,177x4,176x2,175x2,174x2,173x2,172x3,171x3,170x6,",
            "169x2,168,167x2,166x2,165x4,164x2,163x6,162x2,161x3,160x4,159x2,158x5,157x2,156x2,155x2,",
            "154x4,153x3,152x3,151x6,150x2,149x2,148x2,147x2,146x3,145,144x4,143,142x4,141x2,140x3,",
            "139x3,138x4,137x2,136x2,135x2,134x2,133x3,132x5,131,130x3,129x3,128x2,127x2,126x5,125x3,",
            "124x2,123x2,122x3,121x2,120x2,119x6,118x2,117,116x2,115,114x4,113x2,112x4,111x3,110x3,",
            "109x2,108x4,107x3,106,105x3,104x3,103x2,102x2,101x3,100x4,99x2,98x2,97x4,96x2,95x4,94x2,",
            "93x3,92x2,91x4,90x2,89x2,88x2,87x2,86x7,85,84x3,83x4,82x2,81,80x2,79x3,78x2,77x3,76x2,",
            "75x5,74,73x4,72x4,71x3,70x3,69x2,68x4,67x2,66x2,65x2,64,63x5,62x2,61x3,60x2,59x3,58x4,",
            "57x4,56,55x4,54x2,53x3,52x3,51x3,50x2,49x2,48,47x5,46x2,45x2,44x2,43x2,42x2,41x3,40x4,",
            "39x3,38x4,37,36,35x3,34x2,33x2,32x3,31x5,30x3,29x2,28,27x4,26x6,25,24,23,22,21x2,20x3,",
            "19x3,18x4,17x4,16x3,15,14x3,13x4,12,11x3,10x3,9,8x2,7x2",
        ),
    );
    // The multiply walk's round count. Its denominator is as uniformly
    // distributed as the divide's, so it needs the same depth; the inherited
    // two-round discount was paid for out of the same failure budget.
    set_default_env("PP_ROUNDS_MUL", "730");
    // The peak, and therefore half the score. Both carry-ladder sites size
    // themselves against this, so the achieved peak is exactly this value as
    // long as it is at or above the rigid floor. That floor is
    // 512 (numerator + coefficient) + 727 (tape) + 61 (fold ladder) + 8, i.e.
    // 1308, measured with PEAK_CENSUS. Setting the cap below it buys no qubits
    // and costs Toffoli: at 1298 the same build reports 1308 and spends 7,400
    // more gates.
    set_default_env("PP_WALK_MAX_QUBITS", "1308");

    // ── The replay fold window ─────────────────────────────────────────────

    // The truncation window each replay cell folds its modular correction at.
    // The correction is +/-f or +2f with f = 2^32 + 977, so a carry escapes the
    // window with probability about 2^(33-window); calibrated against measured
    // failure counts at windows 45 and 50 the constant is 3.3e16 * 2^-window
    // expected wrong shots per run, giving 0.0036 at the effective 63.
    // The base is 64 and the profile below takes one bit back, which is how the
    // effective 63 is expressed while keeping `fold_ramp_start` at round 0 --
    // that is what frees `PP_R2` to place a trailing batch at all.
    set_default_env("PP_REPLAY_FOLD_WINDOW", "64");
    set_default_env("PP_REPLAY_FOLD_WINDOW_MUL", "64");
    // Flat. The inherited profile narrowed the late rounds against a measured
    // rate that only holds once the walk has converged, which is exactly the
    // case this configuration stops assuming.
    set_default_env("PP_FOLD_PROFILE", "0:-1");
    set_default_env("PP_FOLD_WIDEN", "0");
    // Where the trailing replay batch takes over. This is worth 25 qubits: the
    // rounds in the batch are replayed at the terminal state, where
    // `loan_terminal` has collapsed both walk registers, instead of beside a
    // live walk register. Measured across 600..730 the peak is flat at 1308
    // from 600 to 620 and rises past it.
    set_default_env("PP_R2", "620");

    // ── The two phase-channel compares ─────────────────────────────────────
    //
    // Both repair a measured-out carry by recomputing `a < b` over the top k
    // bits, so each is wrong with probability about 2^-k per call and the error
    // appears as leftover phase. Their Toffoli sit under a `push_condition` on
    // the measurement outcome and so execute on about half the shots, which is
    // why they are the cheapest place in the tree to buy correctness: about 700
    // executed Toffoli per bit against the fold window's 1,480.
    set_default_env("PP_REPLAY_CHUNK_COMPARE", "34");
    set_default_env("PP_CHUNK_SHAPE", "0:0");
    set_default_env("PP_REPLAY_FLAG_COMPARE", "33");
    set_default_env("PP_FLAG_SHAPE", "0:0");

    // ── The two widths every approximation outside the replay reads ────────
    //
    // These govern about fifty truncated constant folds per shot in the
    // coordinate shell and the square, all of which run far below the peak, so
    // widening them costs about 68 Toffoli per bit and no qubits at all. The
    // inherited 21/22 left 0.007 expected wrong shots on the table for nothing.
    set_default_env("FOLD_GUARD", "32");
    set_default_env("ERASE_COMPARE", "32");

    build_point_add()
}
