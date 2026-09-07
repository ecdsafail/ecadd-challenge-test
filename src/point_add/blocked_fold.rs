//! A space/time trade for the entire signed correction ripple.
//!
//! Keep only carries at block boundaries. Each block streams its selected
//! addend through a short exact measurement-uncomputed ripple.
//! After the sum is complete, erase boundary carries from high to low with the
//! FULL producing block comparison, including its still-live incoming carry:
//! carry_out = (sum < addend + carry_in). No new truncated repair is introduced.
use super::*;

pub(super) fn constant(circ: &mut Builder, acc: &[QubitId], c: U256, ctrl: QubitId, block: usize) {
    let zero = circ.alloc_qubit();
    let first = if c.bit(0) { and_clean(circ, ctrl, acc[0]) } else { circ.alloc_qubit() };
    selected(circ, acc, c, ctrl, None, zero, first, block);
    if c.bit(0) {
        circ.cx(ctrl, acc[0]);
        and_uncompute(circ, first, ctrl, acc[0]);
        circ.cx(ctrl, acc[0]);
    } else { circ.free(first); }
    circ.free(zero);
}

pub(super) fn selected(
    circ: &mut Builder, acc: &[QubitId], f: U256, plus_f: QubitId,
    plus_2f: Option<QubitId>, minus_f: QubitId, first_carry: QubitId,
    block: usize,
) {
    assert!(block >= 2);
    assert!(acc.len() >= 3);
    let negative = twos_complement_bits(f, acc.len());
    let xor_operand = |circ: &mut Builder, i: usize, out: QubitId| {
        if f.bit(i) { circ.cx(plus_f, out); }
        if i > 0 && f.bit(i - 1) {
            if let Some(q) = plus_2f { circ.cx(q, out); }
        }
        if negative[i] { circ.cx(minus_f, out); }
    };
    xor_operand(circ, 0, acc[0]);
    let selectors = |i: usize| {
        let mut result=Vec::new();
        if f.bit(i) {result.push(plus_f);}
        if i>0 && f.bit(i-1) {result.extend(plus_2f);}
        if negative[i] {result.push(minus_f);}
        result
    };
    let mut boundaries = Vec::new();
    let mut incoming = first_carry;
    for lo in (1..acc.len()).step_by(block) {
        let hi=(lo+block).min(acc.len());
        let has_out=hi<acc.len();
        let carries=circ.alloc_qubits(hi-lo-usize::from(!has_out));
        let previous=|i:usize| if i==0 {incoming} else {carries[i-1]};
        for (i,&carry) in carries.iter().enumerate() {
            fold_step(circ,acc[lo+i],previous(i),carry,&selectors(lo+i),false);
        }
        let last=hi-1;
        if !has_out {circ.cx(previous(carries.len()),acc[last]);}
        for q in selectors(last) {circ.cx(q,acc[last]);}
        let owned=carries.len()-usize::from(has_out);
        for i in (0..owned).rev() {
            unwind_fold_step(circ,acc[lo+i],previous(i),carries[i],&selectors(lo+i));
        }
        circ.free_vec(&carries[..owned]);
        if has_out {
            let outgoing=*carries.last().unwrap();
            boundaries.push((lo,hi,incoming,outgoing));
            incoming=outgoing;
        }
    }
    for (lo, hi, incoming, outgoing) in boundaries.into_iter().rev() {
        let operand = circ.alloc_qubits(hi - lo);
        for (i, &q) in operand.iter().enumerate() { xor_operand(circ, lo+i, q); }
        erase_with_compare(circ, outgoing, &acc[lo..hi], &operand, Some(incoming));
        circ.free(outgoing);
        for (i, &q) in operand.iter().enumerate() { xor_operand(circ, lo+i, q); }
        circ.free_vec(&operand);
    }
}

#[cfg(test)]
#[path = "blocked_fold_tests.rs"]
mod tests;
