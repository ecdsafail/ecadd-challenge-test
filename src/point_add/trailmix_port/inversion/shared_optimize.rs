//! Exact local simplification of the emitted shared EEA primitive stream.
use crate::circuit::{Op,OperationType,QubitId,NO_BIT};

/// Exact parity factoring for adjacent CCX gates with the same target and
/// one common control. a*b XOR a*c = a*(b XOR c). Restore c afterwards.
/// Saves one Toffoli in exchange for two CNOTs; allocates no quantum wires.
pub(super) fn factor_adjacent_targets(ops:&mut Vec<Op>)->usize{
    let mut out=Vec::with_capacity(ops.len());let mut i=0;let mut saved=0;
    while i<ops.len(){
        if i+1<ops.len(){let a=ops[i];let b=ops[i+1];
            if a.kind==OperationType::CCX&&b.kind==OperationType::CCX&&a.c_condition==NO_BIT&&b.c_condition==NO_BIT&&a.q_target==b.q_target{
                let ac=[a.q_control1,a.q_control2];let bc=[b.q_control1,b.q_control2];
                let shared:Vec<_>=ac.iter().copied().filter(|q|bc.contains(q)).collect();
                if shared.len()==1{
                    let common=shared[0];let x=*ac.iter().find(|&&q|q!=common).unwrap();let y=*bc.iter().find(|&&q|q!=common).unwrap();
                    let mut cx=a;cx.kind=OperationType::CX;cx.q_control1=x;cx.q_control2=crate::circuit::NO_QUBIT;cx.q_target=y;
                    let mut ccx=a;ccx.q_control1=common;ccx.q_control2=y;cx.validate();ccx.validate();
                    out.extend([cx,ccx,cx]);saved+=1;i+=2;continue;
                }
            }
        }
        out.push(ops[i]);i+=1;
    }
    *ops=out;saved
}

/// Exact NCT pair cancellation. A pair of equal involutions can be removed
/// when every intervening gate commutes with it: neither gate writes a control
/// of the other. All gates here have no classical condition or measurement.
pub(super) fn cancel_nct(ops:&mut Vec<Op>,window:usize,passes:usize)->usize {
    let controls=|op:&Op,q:QubitId| (op.kind==OperationType::CX || op.kind==OperationType::CCX) && op.q_control1==q
        || op.kind==OperationType::CCX && op.q_control2==q;
    let same=|a:&Op,b:&Op| a.kind==b.kind && a.q_target==b.q_target &&
        (if a.kind==OperationType::CCX {(a.q_control1==b.q_control1 && a.q_control2==b.q_control2)||(a.q_control1==b.q_control2 && a.q_control2==b.q_control1)}else{a.q_control1==b.q_control1});
    assert!(ops.iter().all(|op|matches!(op.kind,OperationType::X|OperationType::CX|OperationType::CCX) && op.c_condition==NO_BIT));
    let original=ops.len();
    for _ in 0..passes {
        let mut dead=vec![false;ops.len()];let mut removed=0;
        for i in 0..ops.len() {
            if dead[i] {continue;}
            for j in i+1..ops.len().min(i+1+window) {
                if dead[j] {continue;}
                if same(&ops[i],&ops[j]) {dead[i]=true;dead[j]=true;removed+=2;break;}
                if controls(&ops[i],ops[j].q_target) || controls(&ops[j],ops[i].q_target) {break;}
            }
        }
        if removed==0 {break;}
        let mut out=0;for i in 0..ops.len() {if !dead[i] {ops[out]=ops[i];out+=1;}}
        ops.truncate(out);
    }
    original-ops.len()
}

/// Unlike an index window over an array with tombstones, a live window can
/// collapse an arbitrarily long nested inverse pair in one streaming pass.
pub(super) fn cancel_nct_live(ops:&mut Vec<Op>,window:usize)->usize {
    let controls=|op:&Op,q:QubitId| (op.kind==OperationType::CX || op.kind==OperationType::CCX) && op.q_control1==q
        || op.kind==OperationType::CCX && op.q_control2==q;
    let same=|a:&Op,b:&Op| a.kind==b.kind && a.q_target==b.q_target &&
        (if a.kind==OperationType::CCX {(a.q_control1==b.q_control1 && a.q_control2==b.q_control2)||(a.q_control1==b.q_control2 && a.q_control2==b.q_control1)}else{a.q_control1==b.q_control1});
    assert!(ops.iter().all(|op|matches!(op.kind,OperationType::X|OperationType::CX|OperationType::CCX) && op.c_condition==NO_BIT));
    let original=ops.len();let mut live:Vec<Op>=Vec::with_capacity(original);
    for op in ops.drain(..) {
        let mut matched=None;
        for j in (live.len().saturating_sub(window)..live.len()).rev() {
            if same(&op,&live[j]) {matched=Some(j);break;}
            if controls(&op,live[j].q_target) || controls(&live[j],op.q_target) {break;}
        }
        if let Some(j)=matched {live.remove(j);}else{live.push(op);}
    }
    *ops=live;original-ops.len()
}
