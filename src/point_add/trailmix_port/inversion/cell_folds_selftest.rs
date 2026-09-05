//! Real-Simulator checks, including dirty-reset auditing and coupled error phases.
use super::*;
use crate::circuit::{analyze_ops, BitId, Op, OperationType as K, NO_BIT};
use crate::point_add::B;
use crate::sim::Simulator;
use sha3::{digest::{ExtendableOutput, Update, XofReader}, Shake256};

fn rng(label: &[u8]) -> sha3::Shake256Reader {
    let mut h = Shake256::default(); h.update(label); h.finalize_xof()
}

struct Words { values: Vec<u64>, i: usize }
impl XofReader for Words {
    fn read(&mut self, out: &mut [u8]) {
        assert_eq!(out.len(), 8);
        out.copy_from_slice(&self.values[self.i].to_le_bytes()); self.i += 1;
    }
}

pub(super) fn run_ops(b: &B, inputs: &[u64], mode: u8, transcript: Option<usize>,
    coupled: Option<(usize, u64)>) -> (Vec<u64>, u64, u64) {
    let (nq, nb, _, _) = analyze_ops(b.ops.iter());
    let nq = (nq as usize).max(inputs.len()).max(b.next_qubit as usize);
    let nb = nb as usize;
    let mut random = rng(b"cell-folds-measurements-v1");
    let mut h = 0;
    let values: Vec<u64> = b.ops.iter().filter_map(|op| {
        if !matches!(op.kind, K::R | K::Hmr) { return None; }
        let mut bytes = [0; 8]; random.read(&mut bytes);
        let mut value = match mode { 0 => 0, 1 => !0, _ => u64::from_le_bytes(bytes) };
        if op.kind == K::Hmr {
            if let Some(t) = transcript { value = if t >> h & 1 != 0 { !0 } else { 0 }; }
            h += 1;
            if let Some((id, word)) = coupled { if op.q_target.0 as usize == id { value = word; } }
        }
        Some(value)
    }).collect();
    let mut r1 = Words { values: values.clone(), i: 0 };
    let mut r2 = Words { values, i: 0 };
    let mut native = Simulator::new(nq, nb, &mut r1);
    let mut audit = Simulator::new(nq, nb + 1, &mut r2);
    for s in [&mut native, &mut audit] {
        s.qubits[..inputs.len()].copy_from_slice(inputs);
        s.bits.fill(!0);
    }
    native.apply_iter(b.ops.iter());
    let mut active = !0;
    let mut stack = Vec::new();
    for (i, op) in b.ops.iter().enumerate() {
        op.validate();
        match op.kind {
            K::PushCondition => { stack.push(active); active &= audit.bit(op.c_condition); }
            K::PopCondition => { active = stack.pop().unwrap(); }
            _ => {
                if op.kind == K::R { assert_eq!(audit.qubit(op.q_target), 0, "dirty reset {i}: {op:?}"); }
                let mut flat: Op = *op;
                audit.bits[nb] = active & if op.c_condition == NO_BIT { !0 } else { audit.bit(op.c_condition) };
                flat.c_condition = BitId(nb as u64);
                audit.apply_iter(std::iter::once(&flat));
            }
        }
    }
    assert!(stack.is_empty());
    assert_eq!(native.qubits, audit.qubits);
    assert_eq!(native.phase, audit.phase);
    assert_eq!(native.stats, audit.stats);
    assert!(native.qubits[inputs.len()..].iter().all(|&q| q == 0), "scratch");
    (native.qubits[..inputs.len()].to_vec(), native.phase, native.stats.toffoli_gates)
}

fn build_const(n: usize, k: usize, value: &[u8], subtract: bool, old: bool, pad: usize) -> B {
    let mut c = Circuit::new();
    let a = c.alloc_qreg_bits("test.a", n);
    let ctrl = c.alloc_qreg("test.ctrl");
    let dirty = c.alloc_qreg_bits("test.dirty", n.saturating_sub(1));
    let _pad = c.alloc_qreg_bits("test.pad", pad);
    if subtract { for q in &a { c.x(q); } }
    if old {
        crate::point_add::trailmix_port::arith::gidney_const_adder::controlled_add_const_gidney(&mut c, &ctrl, &a, value, &dirty);
    } else { add(&mut c, &a, &ctrl, value, k); }
    if subtract { for q in &a { c.x(q); } }
    c.flush_pending_frees(); c.into_builder()
}

fn exhaustive() -> usize {
    let mut checked = 0;
    for n in 1usize..=7 {
        let mask = (1usize << n) - 1;
        for k in 1..=n {
            for constant in 0..=mask {
                for subtract in [false, true] {
                    let b = build_const(n, k, &[constant as u8], subtract, false, 0);
                    assert_eq!(b.peak_qubits as usize, 2*n + scratch(n, k));
                    for first in (0..1usize << (n+1)).step_by(64) {
                        let input: Vec<u64> = (0..2*n).map(|i| (0..64).fold(0, |word, s|
                            word | ((((first+s).rotate_left(3) ^ (first+s)) >> (i % (n+1)) & 1) as u64) << s)).collect();
                        // Enumerate target/control exactly; dirty donors vary independently.
                        let mut input = input;
                        for i in 0..=n { input[i] = (0..64).fold(0, |word, s| word | ((((first+s) >> i)&1) as u64) << s); }
                        let mut want = input.clone();
                        for i in 0..n { want[i] = 0; }
                        for shot in 0..64 {
                            let old = (first+shot) & mask;
                            let delta = if (first+shot) >> n & 1 != 0 { constant } else { 0 };
                            let value = if subtract { old.wrapping_sub(delta) & mask } else { (old+delta)&mask };
                            for i in 0..n { want[i] |= (((value>>i)&1) as u64) << shot; }
                        }
                        for mode in 0..3 {
                            let (actual, phase, _) = run_ops(&b, &input, mode, None, None);
                            assert_eq!((actual, phase), (want.clone(), 0), "n={n} k={k} const={constant} sub={subtract} mode={mode}");
                            checked += 64;
                        }
                        if n <= 3 {
                            let hmr = b.ops.iter().filter(|op| op.kind == K::Hmr).count();
                            for t in 0..1usize << hmr {
                                let (actual, phase, _) = run_ops(&b, &input, 0, Some(t), None);
                                assert_eq!((actual, phase), (want.clone(), 0)); checked += 64;
                            }
                        }
                    }
                }
            }
        }
    }
    checked
}

fn build_cell(old: bool, inverse: bool, pad: usize) -> B {
    let mut c = Circuit::new();
    let target = c.alloc_qreg_bits("test.target", 257);
    let source = c.alloc_qreg_bits("test.source", 257);
    let sign = c.alloc_qreg("test.sign");
    let _pad = c.alloc_qreg_bits("test.pad", pad);
    if old {
        std::env::set_var("MIDQ_CELL_FOLDS", "0");
        super::super::midq_mod_signed_add_halve(&mut c, &target, &source, &sign, inverse);
    } else { apply(&mut c, &target, &source, &sign, inverse); }
    c.flush_pending_frees(); c.into_builder()
}

fn exhaustive_sum() -> usize {
    let mut checked = 0;
    for n in 1usize..=7 {
        let mask = (1usize << n) - 1;
        for k in 1..=n {
            let mut c = Circuit::new();
            let a = c.alloc_qreg_bits("test.a", n);
            let b = c.alloc_qreg_bits("test.b", n);
            let out = c.alloc_qreg("test.overflow");
            emit(&mut c, &a, Addend::Word(&b), Some(&out), k);
            c.flush_pending_frees();
            let built = c.into_builder();
            assert_eq!(built.peak_qubits as usize, 2*n+1+scratch(n+1, k));
            for first in (0..1usize << (2*n+1)).step_by(64) {
                let input: Vec<u64> = (0..2*n+1).map(|i| (0..64).fold(0, |v, s| v | ((((first+s)>>i)&1) as u64)<<s)).collect();
                let mut want = input.clone();
                want[..n].fill(0); want[2*n] = 0;
                for shot in 0..64 {
                    let x = first+shot;
                    let sum = (x & mask) + ((x >> n)&mask);
                    for i in 0..n { want[i] |= (((sum>>i)&1) as u64)<<shot; }
                    want[2*n] |= ((((sum>>n)^(x>>(2*n)))&1) as u64)<<shot;
                }
                for mode in 0..3 {
                    let (actual, phase, _) = run_ops(&built, &input, mode, None, None);
                    assert_eq!((actual, phase), (want.clone(), 0), "sum n={n} k={k}");
                    checked += 64;
                }
                if n <= 3 {
                    let hmr = built.ops.iter().filter(|op| op.kind == K::Hmr).count();
                    for t in 0..1usize << hmr {
                        let (actual, phase, _) = run_ops(&built, &input, 0, Some(t), None);
                        assert_eq!((actual, phase), (want.clone(), 0)); checked += 64;
                    }
                }
            }
        }
    }
    checked
}

fn full_width() -> usize {
    use alloy_primitives::U256;
    let mut checked = 0;
    let mut random = rng(b"cell-folds-inputs-v1");
    for pad in [0, 445, 465, 470, 485] {
        for inverse in [false, true] {
            let old = build_cell(true, inverse, pad);
            let new = build_cell(false, inverse, pad);
            assert!(new.peak_qubits <= 1009.max(old.peak_qubits));
            let emitted = |b: &B| b.ops.iter().filter(|op| matches!(op.kind, K::CCX | K::CCZ)).count();
            let mut totals = [0, 0];
            for batch in 0..32 {
                let mut input = vec![0; 515+pad];
                for word in &mut input {
                    let mut bytes = [0; 8]; random.read(&mut bytes); *word = u64::from_le_bytes(bytes);
                }
                // Include 0, all ones, powers of two, and long carries in both
                // registers. The overflow wires are deliberately unrestricted.
                for i in 0..256 {
                    input[i] = (input[i] & !15) | 2 | (if i == batch*8 { 4 } else { 0 });
                    input[257+i] = (input[257+i] & !15) | 8 | (if i < batch*8 { 4 } else { 0 });
                }
                if batch >= 16 {
                    let f = U256::from((1u64 << 32) + 977);
                    let p = U256::MAX - f + U256::from(1);
                    let edge = [U256::ZERO, U256::from(1), f-U256::from(1), f, f+U256::from(1), p-U256::from(1), p, U256::MAX];
                    for i in 0..256 {
                        input[i] = (0..64).fold(0, |v,s| v | (u64::from(edge[s%8].bit(i))<<s));
                        input[257+i] = (0..64).fold(0, |v,s| v | (u64::from(edge[s/8].bit(i))<<s));
                    }
                    input[514] = if batch % 2 == 0 { 0 } else { !0 };
                }
                for mode in 0..3 {
                    let coupled = Some((256, if mode == 0 { 0 } else if mode == 1 { !0 } else { 0xa5a5a5a5a5a5a5a5 }));
                    let (v0, p0, t0) = run_ops(&old, &input, mode, None, coupled);
                    let (v1, p1, t1) = run_ops(&new, &input, mode, None, coupled);
                    assert_eq!((v0,p0), (v1,p1), "full-width cell inverse={inverse} pad={pad} batch={batch} mode={mode}");
                    if mode == 2 { totals[0] += t0; totals[1] += t1; }
                    checked += 64;
                }
            }
            eprintln!("CELL_FOLDS_RESOURCE inverse={inverse} pad={pad} oldQ={} newQ={} old_emitted={} new_emitted={} old_avgT={:.3} new_avgT={:.3}", old.peak_qubits, new.peak_qubits, emitted(&old), emitted(&new), totals[0] as f64 / 2048.0, totals[1] as f64 / 2048.0);
        }
    }
    checked
}

pub(crate) fn run() {
    clean_chunk_plan::selftest();
    std::env::set_var("MIDQ_COMPACT_CONST_CARRY", "1");
    std::env::set_var("MIDQ_DIRTY_CONST", "1");
    std::env::set_var("MIDQ_MEASURE_COMPARE", "1");
    std::env::set_var("MIDQ_CHUNK_COMPARE", "1");
    std::env::set_var("MIDQ_CELL_QCAP", "1009");
    let small = exhaustive();
    eprintln!("CELL_FOLDS_EXHAUSTIVE PASS {small} cases, all small measurement transcripts");
    let sum = exhaustive_sum();
    eprintln!("CELL_FOLDS_SUM PASS {sum} cases, arbitrary overflow bit, all small measurement transcripts");
    let full = full_width();
    eprintln!("MIDQ_CELL_FOLDS_SELFTEST PASS: {} cases, exact finite-width values, coupled noncanonical error phases, donor restoration and pre-reset audit", small + sum + full);
}
