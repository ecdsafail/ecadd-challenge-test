//! Controlled addition with bounded carries and exact post-sum cleanup.
use crate::point_add::trailmix_port::circuit::{Circuit, QReg};

fn majority(c: &mut Circuit, a: &QReg, b: &QReg, previous: Option<&QReg>,
    out: &QReg, complement: Option<&QReg>) {
    if let Some(g) = complement { c.cx(g, a); }
    if let Some(p) = previous {
        c.cx(p, a); c.cx(p, b);
        c.ccx(a, b, out);
        c.cx(p, out);
        c.cx(p, b); c.cx(p, a);
    } else { c.ccx(a, b, out); }
    if let Some(g) = complement { c.cx(g, a); }
}

fn phase(c: &mut Circuit, a: &QReg, b: &QReg, previous: Option<&QReg>, g: &QReg) {
    // If s=a XOR g*(b XOR carry), MAJ(a,b,carry)=MAJ(s XOR g,b,carry).
    c.cx(g, a);
    c.cz(a, b);
    if let Some(p) = previous { c.cz(a, p); c.cz(b, p); }
    c.cx(g, a);
}

fn clear(c: &mut Circuit, out: &QReg, a: &QReg, b: &QReg, previous: Option<&QReg>, g: &QReg) {
    let m = c.alloc_bit();
    c.hmr(out, m);
    c.with_condition(m, |c| phase(c, a, b, previous, g));
    c.free_bit(m);
}

fn sum(c: &mut Circuit, a: &QReg, b: &QReg, previous: Option<&QReg>, g: &QReg) {
    if let Some(p) = previous { c.cx(p, b); }
    c.ccx(g, b, a);
    if let Some(p) = previous { c.cx(p, b); }
}

pub(crate) fn try_apply(c: &mut Circuit, g: &QReg, a: &[&QReg], b: &[&QReg], budget: usize) -> bool {
    if std::env::var("MIDQ_CHUNKED_CONTROLLED_ADD").ok().as_deref() != Some("1")
        || a.len() < 2 { return false; }
    assert_eq!(a.len(), b.len());
    let mut ids = std::collections::HashSet::new();
    if a.iter().chain(b).any(|q| q.id() == g.id() || !ids.insert(q.id())) { return false; }
    c.flush_pending_frees();
    let cap = std::env::var("MIDQ_CONTROLLED_ADD_QCAP").ok()
        .and_then(|v| v.parse::<usize>().ok()).unwrap_or(1009);
    let available = budget.min(cap.saturating_sub(c.b.active_qubits as usize));
    let count = a.len() - 1;
    // The old fully vented network already attains the same Toffoli count.
    if available >= count { return false; }
    let Some(chunks) = super::clean_chunk_plan::plan(count, available) else { return false; };
    emit(c, g, a, b, &chunks);
    true
}

fn emit(c: &mut Circuit, g: &QReg, a: &[&QReg], b: &[&QReg], chunks: &[usize]) {
    let section = c.push_section("chunked.controlled_add");
    let count = a.len() - 1;
    let mut boundaries: Vec<QReg> = Vec::new();
    let mut ranges = Vec::new();
    let mut start = 0;
    for &size in chunks {
        let end = start + size;
        let initial = boundaries.last();
        let mut chain: Vec<QReg> = Vec::new();
        for i in start..end {
            let next = c.alloc_qreg("cadd.carry");
            let previous = chain.last().or(initial);
            majority(c, a[i], b[i], previous, &next, None);
            sum(c, a[i], b[i], previous, g);
            chain.push(next);
        }
        let keep = end < count;
        if !keep { sum(c, a[count], b[count], chain.last(), g); }
        for j in (0..chain.len() - usize::from(keep)).rev() {
            let previous = if j == 0 { initial } else { Some(&chain[j - 1]) };
            clear(c, &chain[j], a[start + j], b[start + j], previous, g);
        }
        let retained = keep.then(|| chain.pop().unwrap());
        for bit in chain { c.zero_and_free(bit); }
        if let Some(bit) = retained { boundaries.push(bit); ranges.push((start, end)); }
        start = end;
    }
    assert_eq!(start, count);
    while let Some(boundary) = boundaries.pop() {
        let (start, end) = ranges.pop().unwrap();
        let initial = boundaries.last();
        let m = c.alloc_bit();
        c.hmr(&boundary, m);
        c.zero_and_free(boundary);
        c.with_condition(m, |c| {
            let mut chain: Vec<QReg> = Vec::new();
            for i in start..end - 1 {
                let next = c.alloc_qreg("cadd.phase.carry");
                majority(c, a[i], b[i], chain.last().or(initial), &next, Some(g));
                chain.push(next);
            }
            phase(c, a[end - 1], b[end - 1], chain.last().or(initial), g);
            for j in (0..chain.len()).rev() {
                let previous = if j == 0 { initial } else { Some(&chain[j - 1]) };
                clear(c, &chain[j], a[start + j], b[start + j], previous, g);
            }
            for bit in chain { c.zero_and_free(bit); }
        });
        c.free_bit(m);
    }
    c.pop_section(&section);
}

#[path = "chunked_controlled_add_selftest.rs"]
mod tests;
pub(crate) use tests::run as selftest;
