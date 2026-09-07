//! Rank5 physical inversion lifecycle. Complete point-add validation is separate.
use crate::point_add::trailmix_port::circuit::{Circuit,QReg};
use crate::circuit::{Op,OperationType,NO_QUBIT,QubitId};
const FIELD_WIDTH:usize=257;const VALUE_WIDTH:usize=256;const WORK_WIDTH:usize=259;
const HALF_PLUS_ONE_LE:[u8;33]={let mut b=[0xff;33];b[0]=0x18;b[1]=0xfe;b[3]=0x7f;b[31]=0x7f;b[32]=0;b};
struct Core {rank:Vec<QReg>,a:Vec<QReg>,c:Vec<QReg>,sm:Vec<QReg>,phase1:QReg,phase2:QReg,sign:QReg,iteration:QReg,work1:Vec<QReg>,work2:Vec<QReg>}
struct Terminal {iteration:QReg,work2:Vec<QReg>,history:Vec<QReg>}
fn free_clean(circ: &mut Circuit, register: Vec<QReg>) {
    for lane in register {
        circ.zero_and_free(lane);
    }
}

fn toggle_constant(circ: &mut Circuit, register: &[QReg], value: usize) {
    for (index, lane) in register.iter().enumerate() {
        if (value >> index) & 1 != 0 {
            circ.x(lane);
        }
    }
}

fn toggle_initial_work1(circ: &mut Circuit, work1: &[QReg]) {
    use crate::point_add::trailmix_port::mod_arith::SECP256K1_P_LE;

    assert_eq!(work1.len(), WORK_WIDTH);
    circ.x(&work1[0]);
    for bit in 0..VALUE_WIDTH {
        if (SECP256K1_P_LE[bit / 8] >> (bit % 8)) & 1 != 0 {
            circ.x(&work1[WORK_WIDTH - 1 - bit]);
        }
    }
}

fn toggle_terminal_work1(circ: &mut Circuit, work1: &[QReg]) {
    use crate::point_add::trailmix_port::mod_arith::SECP256K1_P_LE;

    assert_eq!(work1.len(), WORK_WIDTH);
    for bit in 0..VALUE_WIDTH {
        if (SECP256K1_P_LE[bit / 8] >> (bit % 8)) & 1 != 0 {
            circ.x(&work1[bit]);
        }
    }
    circ.x(&work1[WORK_WIDTH - 1]);
}


fn ids(core:&Core)->Vec<usize> {
    let mut out:Vec<_>=core.rank.iter().chain(&core.a).chain(&core.c).chain(&core.sm).map(|q|q.id()as usize).collect();
    out.extend([core.phase1.id(),core.phase2.id(),core.sign.id(),core.iteration.id()].into_iter().map(|i|i as usize));
    out.extend(core.work1.iter().chain(&core.work2).map(|q|q.id()as usize));assert_eq!(out.len(),543);out
}
fn initialize(circ:&mut Circuit,mut dx:Vec<QReg>)->Core {
    use super::shrunken_pz_state_machine::controlled_field_neg;
    use crate::point_add::trailmix_port::arith::compare::compare_geq_const;
    assert_eq!(dx.len(),257);let iteration=circ.alloc_qreg("rank5.iteration");
    compare_geq_const(circ,&dx,&HALF_PLUS_ONE_LE,&iteration);controlled_field_neg(circ,&iteration,&dx);
    dx.push(circ.alloc_qreg("rank5.work2-pad0"));dx.push(circ.alloc_qreg("rank5.work2-pad1"));dx.reverse();
    let work1=circ.alloc_qreg_bits("rank5.work1",259);toggle_initial_work1(circ,&work1);
    Core {rank:circ.alloc_qreg_bits("rank5.rank",5),a:circ.alloc_qreg_bits("rank5.a",6),c:circ.alloc_qreg_bits("rank5.c",6),sm:circ.alloc_qreg_bits("rank5.sm",4),phase1:circ.alloc_qreg("rank5.P1"),phase2:circ.alloc_qreg("rank5.P2"),sign:circ.alloc_qreg("rank5.Sign"),iteration,work1,work2:dx}
}
fn release_terminal(circ:&mut Circuit,mut core:Core)->Terminal {
    toggle_terminal_work1(circ,&core.work1);free_clean(circ,core.work1);
    toggle_constant(circ,&core.rank,29);free_clean(circ,core.rank);toggle_constant(circ,&core.a,63);free_clean(circ,core.a);
    let high=core.sm.split_off(2);free_clean(circ,high);let mut history=core.c;history.extend(core.sm);assert_eq!(history.len(),8);
    circ.zero_and_free(core.phase1);circ.zero_and_free(core.phase2);circ.zero_and_free(core.sign);
    Terminal {iteration:core.iteration,work2:core.work2,history}
}
fn rebuild_terminal(circ:&mut Circuit,mut terminal:Terminal)->Core {
    let work1=circ.alloc_qreg_bits("rank5.work1.rebuilt",259);toggle_terminal_work1(circ,&work1);
    let rank=circ.alloc_qreg_bits("rank5.rank.rebuilt",5);toggle_constant(circ,&rank,29);
    let a=circ.alloc_qreg_bits("rank5.a.rebuilt",6);toggle_constant(circ,&a,63);
    let mut sm=terminal.history.split_off(6);sm.extend(circ.alloc_qreg_bits("rank5.sm.high.rebuilt",2));
    Core {rank,a,c:terminal.history,sm,phase1:circ.alloc_qreg("rank5.P1.rebuilt"),phase2:circ.alloc_qreg("rank5.P2.rebuilt"),sign:circ.alloc_qreg("rank5.Sign.rebuilt"),iteration:terminal.iteration,work1,work2:terminal.work2}
}
fn finish(circ:&mut Circuit,mut core:Core)->Vec<QReg> {
    use super::shrunken_pz_state_machine::controlled_field_neg;
    use crate::point_add::trailmix_port::arith::compare::compare_geq_const;
    circ.zero_and_free(core.phase1);circ.zero_and_free(core.phase2);circ.zero_and_free(core.sign);
    toggle_initial_work1(circ,&core.work1);free_clean(circ,core.work1);free_clean(circ,core.rank);free_clean(circ,core.a);free_clean(circ,core.c);free_clean(circ,core.sm);
    core.work2.reverse();circ.zero_and_free(core.work2.pop().unwrap());circ.zero_and_free(core.work2.pop().unwrap());assert_eq!(core.work2.len(),257);
    controlled_field_neg(circ,&core.iteration,&core.work2);compare_geq_const(circ,&core.work2,&HALF_PLUS_ONE_LE,&core.iteration);circ.zero_and_free(core.iteration);core.work2
}
fn toggle_inverse_sign(circ:&mut Circuit,terminal:&Terminal) {
    use super::shrunken_pz_state_machine::controlled_field_neg;
    circ.x(&terminal.iteration);controlled_field_neg(circ,&terminal.iteration,&terminal.work2[..257]);circ.x(&terminal.iteration);
}
/// Logical primitive template; physical remapping preserves every alias relation.
fn template(block:usize,j:usize)->Vec<Op> {
    let mut circ=Circuit::new();circ.b.count_only=false;circ.b.fiat_hash=None;let rank=circ.alloc_qreg_bits("rank",5);let a=circ.alloc_qreg_bits("a",6);let c=circ.alloc_qreg_bits("c",6);let sm=circ.alloc_qreg_bits("sm",4);
    let p1=circ.alloc_qreg("p1");let p2=circ.alloc_qreg("p2");let sign=circ.alloc_qreg("sign");let iter=circ.alloc_qreg("iter");let w1=circ.alloc_qreg_bits("w1",259);let w2=circ.alloc_qreg_bits("w2",259);let helpers=circ.alloc_qreg_bits("borrowed",24);assert_eq!(circ.b.next_qubit,567);
    super::metadata_full_step5::step(&mut circ,&rank,&a,&c,&sm,&p1,&p2,&sign,&iter,&w1,&w2,&helpers,j,block);assert_eq!(circ.b.next_qubit,567);
    let b=circ.into_builder();assert!(b.ops.iter().all(|o|matches!(o.kind,OperationType::X|OperationType::CX|OperationType::CCX)));b.ops
}
fn remap(mut ops:Vec<Op>,core_ids:&[usize],passenger:&[QReg],inverse:bool)->Vec<Op> {
    assert_eq!(core_ids.len(),543);assert!(passenger.len()>=24);let mut mapping=core_ids.to_vec();mapping.extend(passenger[..24].iter().map(|q|q.id()as usize));
    let mut unique=mapping.clone();unique.sort_unstable();unique.dedup();assert_eq!(unique.len(),567);
    for op in &mut ops {for q in [&mut op.q_control1,&mut op.q_control2,&mut op.q_target] {if *q!=NO_QUBIT{*q=QubitId(mapping[q.0 as usize]as u64);}}op.validate();}
    if inverse{ops.reverse();}ops
}
#[path="rank5_lifecycle_check.rs"]pub mod check;

struct CanonicalTopLoan {
    restored: bool,
    context: &'static str,
}

impl Drop for CanonicalTopLoan {
    fn drop(&mut self) {
        assert!(
            self.restored || std::thread::panicking(),
            "{} canonical top loan dropped without restore",
            self.context
        );
    }
}

/// Lend a canonical field register's known-zero extension lane to the EEA.
/// The replacement lane need not retain physical identity because the 257th
/// lane is internal, canonical zero state rather than ABI-visible data.
fn loan_canonical_top(
    circ: &mut Circuit,
    register: &mut Vec<QReg>,
    context: &'static str,
) -> CanonicalTopLoan {
    assert_eq!(
        register.len(),
        FIELD_WIDTH,
        "{context} canonical register width"
    );
    let live_before = circ.b.active_qubits;
    let top = register.pop().expect("canonical top lane");
    circ.zero_and_free(top);
    assert_eq!(register.len(), FIELD_WIDTH - 1);
    assert_eq!(
        circ.b.active_qubits + 1,
        live_before,
        "{context} canonical top loan must free one qubit"
    );
    circ.lowq_passenger_top_releases += 1;
    CanonicalTopLoan {
        restored: false,
        context,
    }
}

fn restore_canonical_top(circ: &mut Circuit, register: &mut Vec<QReg>, mut loan: CanonicalTopLoan) {
    assert_eq!(
        register.len(),
        FIELD_WIDTH - 1,
        "{} shortened canonical register width",
        loan.context
    );
    let live_before = circ.b.active_qubits;
    register.push(circ.alloc_qreg(&format!("{}.restored", loan.context)));
    assert_eq!(register.len(), FIELD_WIDTH);
    assert_eq!(
        circ.b.active_qubits,
        live_before + 1,
        "{} canonical top restore must allocate one clean qubit",
        loan.context
    );
    assert!(
        circ.lowq_passenger_top_releases > 0,
        "passenger top loan state underflow"
    );
    circ.lowq_passenger_top_releases -= 1;
    loan.restored = true;
}


pub fn enabled()->bool {std::env::var("LOWQ_RANK5_EEA").ok().as_deref()==Some("1")}
fn emit_schedule(circ:&mut Circuit,core:&Core,passenger:&[QReg],inverse:bool) {
    assert!(circ.b.count_only_capture_stack.is_empty());circ.b.flush_compact_block();let mapping=ids(core);
    for z in 0..26 {let block=if inverse{25-z}else{z};let mut cached=Vec::new();
        for j in 0..4 {let ops=remap(template(block,j),&mapping,passenger,inverse);let mut kinds=[0usize;18];for op in &ops{kinds[op.kind as usize]+=1;}cached.push((std::sync::Arc::new(ops),kinds));}
        let first=block*64;let end=(first+64).min(1616);
        for i in 0..end-first {let step=if inverse{end-1-i}else{first+i};let (ops,kinds)=&cached[(step+1)%4];
            if circ.b.compact_blocks.is_some(){assert!(circ.b.ops.is_empty());circ.b.compact_blocks.as_mut().unwrap().push(ops.clone());for kind in [OperationType::X,OperationType::CX,OperationType::CCX]{circ.b.add_counted_kind(kind,kinds[kind as usize]);}}
            else if circ.b.count_only&&circ.b.fiat_hash.is_none(){for kind in [OperationType::X,OperationType::CX,OperationType::CCX]{circ.b.add_counted_kind(kind,kinds[kind as usize]);}}
            else{for &op in ops.iter(){circ.b.push_op(op);}}
        }
    }
}
fn emit_forward(circ:&mut Circuit,core:&Core,passenger:&[QReg]){emit_schedule(circ,core,passenger,false);}
fn emit_reverse(circ:&mut Circuit,core:&Core,passenger:&[QReg]){emit_schedule(circ,core,passenger,true);}
fn canonicalize_terminal_work2(_circ:&mut Circuit,_terminal:&Terminal){}
fn restore_terminal_work2_rotation(_circ:&mut Circuit,_terminal:&Terminal){}

// Canonical Work2 is coefficient ||000. Keep the257-bit field view and
// temporarily release its two unused high zeros during outer multiplication.
fn release_terminal_padding(circ:&mut Circuit,terminal:&mut Terminal){
    assert_eq!(terminal.work2.len(),259);for _ in 0..2{circ.zero_and_free(terminal.work2.pop().unwrap());}
}
fn restore_terminal_padding(circ:&mut Circuit,terminal:&mut Terminal){
    assert_eq!(terminal.work2.len(),257);terminal.work2.extend(circ.alloc_qreg_bits("rank5.terminal-padding.restored",2));
}
pub fn divide_forward(
    circ: &mut Circuit,
    dx: Vec<QReg>,
    mut dy: Vec<QReg>,
) -> (Vec<QReg>, Vec<QReg>, Vec<QReg>) {
    use super::shrunken_pz_state_machine::{
        release_q955_canonical_lambda_top, restore_q955_canonical_lambda_top,
    };
    use crate::point_add::trailmix_port::arith::rfold_mbu::mod_mul_canonical_mbu;

    assert_eq!(dx.len(), FIELD_WIDTH);
    assert_eq!(dy.len(), FIELD_WIDTH);
    let released_dy_top = loan_canonical_top(circ, &mut dy, "paper2607 forward dy");
    let core = initialize(circ, dx);
    emit_forward(circ, &core, &dy);
    let mut terminal = release_terminal(circ, core);
    release_terminal_padding(circ,&mut terminal);
    canonicalize_terminal_work2(circ, &terminal);
    toggle_inverse_sign(circ, &terminal);

    restore_canonical_top(circ, &mut dy, released_dy_top);
    let mut lambda = circ.alloc_qreg_bits("paper2607.lambda", FIELD_WIDTH);
    mod_mul_canonical_mbu(circ, &lambda, &terminal.work2[..FIELD_WIDTH], &dy);
    toggle_inverse_sign(circ, &terminal);
    restore_terminal_work2_rotation(circ, &terminal);
    release_q955_canonical_lambda_top(circ, &mut lambda);

    let dy_ghosts: Vec<_> = dy.iter().map(|lane| circ.hmr_ghost(lane)).collect();
    free_clean(circ, dy);
    restore_terminal_padding(circ,&mut terminal);
    let core = rebuild_terminal(circ, terminal);
    emit_reverse(circ, &core, &lambda);
    let dx = finish(circ, core);

    restore_q955_canonical_lambda_top(circ, &mut lambda);
    let dy = circ.alloc_qreg_bits("paper2607.dy-restored", FIELD_WIDTH);
    mod_mul_canonical_mbu(circ, &dy, &lambda, &dx);
    for (ghost, lane) in dy_ghosts.into_iter().zip(&dy) {
        circ.resolve_ghost(ghost, lane);
    }
    (dx, dy, lambda)
}

pub fn divide_cancel(
    circ: &mut Circuit,
    dx: Vec<QReg>,
    mut dy: Vec<QReg>,
    lambda: Vec<QReg>,
) -> (Vec<QReg>, Vec<QReg>) {
    use crate::point_add::trailmix_port::arith::rfold_mbu::{
        mod_mul_canonical_mbu, mod_mul_canonical_mbu_undo,
    };

    assert_eq!(dx.len(), FIELD_WIDTH);
    assert_eq!(dy.len(), FIELD_WIDTH);
    assert_eq!(lambda.len(), FIELD_WIDTH);
    let lambda_ghosts: Vec<_> = lambda.iter().map(|lane| circ.hmr_ghost(lane)).collect();
    free_clean(circ, lambda);

    let released_forward_dy_top = loan_canonical_top(circ, &mut dy, "paper2607 cancel-forward dy");
    let core = initialize(circ, dx);
    emit_forward(circ, &core, &dy);
    let mut terminal = release_terminal(circ, core);
    release_terminal_padding(circ,&mut terminal);
    canonicalize_terminal_work2(circ, &terminal);
    toggle_inverse_sign(circ, &terminal);

    restore_canonical_top(circ, &mut dy, released_forward_dy_top);
    let quotient = circ.alloc_qreg_bits("paper2607.quotient-check", FIELD_WIDTH);
    mod_mul_canonical_mbu(circ, &quotient, &terminal.work2[..FIELD_WIDTH], &dy);
    for (ghost, lane) in lambda_ghosts.into_iter().zip(&quotient) {
        circ.resolve_ghost(ghost, lane);
    }
    mod_mul_canonical_mbu_undo(circ, &quotient, &terminal.work2[..FIELD_WIDTH], &dy);
    free_clean(circ, quotient);

    toggle_inverse_sign(circ, &terminal);
    restore_terminal_work2_rotation(circ, &terminal);
    let released_reverse_dy_top = loan_canonical_top(circ, &mut dy, "paper2607 cancel-reverse dy");
    restore_terminal_padding(circ,&mut terminal);
    let core = rebuild_terminal(circ, terminal);
    emit_reverse(circ, &core, &dy);
    let dx = finish(circ, core);
    restore_canonical_top(circ, &mut dy, released_reverse_dy_top);
    (dx, dy)
}

