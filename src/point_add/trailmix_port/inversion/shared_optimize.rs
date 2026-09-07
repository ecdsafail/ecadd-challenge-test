//! Exact local simplification of the emitted shared EEA primitive stream.
use crate::circuit::{Op,OperationType,QubitId,NO_BIT};

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
