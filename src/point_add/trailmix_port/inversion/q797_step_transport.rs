//! Complete decoded EEA step carrying a missing phase lane's arbitrary cargo.
//! Component only: boundary six-bit encoder and lifecycle remain separate.
use crate::point_add::trailmix_port::circuit::{Circuit,QReg};
use super::{q798_step as old,q797_cargo_moves as moves,length_recompute::mixed_mcx};
#[path="metadata_phase115_programs.rs"] mod programs;
type Terms<'a>=Vec<Vec<(&'a QReg,bool)>>;
fn ceq<'a>(rank:&'a[QReg],c:&'a[QReg],value:usize,base:&[(&'a QReg,bool)])->Terms<'a>{
    programs::C_EQUAL[value>>6].iter().map(|&(m,v)|{let mut cs=base.to_vec();cs.extend(c.iter().enumerate().map(|(i,q)|(q,value>>i&1!=0)));cs.extend((0..5).filter(|&i|m>>i&1!=0).map(|i|(&rank[i],v>>i&1!=0)));cs}).collect()
}
fn product<'a>(a:&Terms<'a>,b:&Terms<'a>)->Terms<'a>{
    let mut out=Vec::new();for x in a{for y in b{let mut z=x.clone();let mut valid=true;for &(q,v)in y{if let Some(&(_,old))=z.iter().find(|&&(p,_)|p.id()==q.id()){if old!=v{valid=false;break;}}else{z.push((q,v));}}if valid{out.push(z);}}}out
}
fn szero<'a>(rank:&'a[QReg],c:&'a[QReg],sm:&'a[QReg],j:usize)->Terms<'a>{
    if j%2==1{return vec![];}
    programs::S_ZERO[0].iter().map(|&(m,v)|{let mut cs=vec![(&c[0],j==2)];cs.extend(sm.iter().map(|q|(q,false)));cs.extend((0..5).filter(|&i|m>>i&1!=0).map(|i|(&rank[i],v>>i&1!=0)));cs}).collect()
}
fn newborn(circ:&mut Circuit,rank:&[QReg],a:&[QReg],c:&[QReg],sm:&[QReg],p2:&QReg,sign:&QReg,w1:&[QReg],w2:&[QReg],dirty:&[QReg],j:usize){
    circ.cx(sign,p2); // P2 is now a clean carry cache on newborn phase11.
    super::metadata_phase115_phased::prepare(circ,c,sm,sign,Some(p2),dirty,j,false);
    let start=circ.b.ops.len();let mut nodes=vec![None;512];for value in 1..=257{nodes[value]=Some(&w2[259-value]);}
    let ts:Vec<_>=(0..4).flat_map(|a|(0..4).flat_map(move|c|(0..4).filter(move|&s|a+c+s<=4).map(move|s|[a,c,s]))).collect();
    for level in 0..9{let mut next=Vec::new();for pair in nodes.chunks_exact(2){next.push(match(pair[0],pair[1]){
        (Some(left),Some(right))=>{
            if level<6{circ.cswap(&c[level],left,right);}else{
                let ctrl:Vec<_>=rank.iter().chain(std::iter::once(p2)).collect();let truth:Vec<_>=(0..64).map(|r|((ts[r&31][1]+ts[r&31][2]+(r>>5))>>(level-6))&1!=0).collect();
                super::metadata_muxlease::truth_swap(circ,&ctrl,truth,left,right,dirty);
                // On a newborn S_raw0 at j0 the actual S is256, not0.
                if level==8&&j==0{let mut cs=vec![(sign,true)];cs.extend(rank.iter().chain(sm).map(|q|(q,false)));circ.cx(right,left);cs.push((left,true));mixed_mcx(circ,&cs,right,dirty);circ.cx(right,left);}
            }Some(left)
        },(Some(q),None)|(None,Some(q))=>Some(q),(None,None)=>None,
    });}nodes=next;}
    let right=nodes[0].unwrap();let r=circ.b.ops[start..].to_vec();let(left,l)=super::q798_handoffs::gather_a(circ,rank,a,w1,3,dirty);
    circ.cswap(sign,left,right);circ.cx(sign,left); // restore the old zero gap
    circ.b.ops.extend(l.into_iter().rev());circ.b.ops.extend(r.into_iter().rev());
    super::metadata_phase115_phased::prepare(circ,c,sm,sign,Some(p2),dirty,j,true);circ.cx(sign,p2);
}
pub(super) fn step(circ:&mut Circuit,rank:&[QReg],a:&[QReg],c:&[QReg],sm:&[QReg],p1:&QReg,p2:&QReg,iteration:&QReg,w1:&[QReg],w2:&[QReg],helpers:&[QReg],post_j:usize,block:usize){
    assert_eq!(helpers.len(),23);let start=circ.b.ops.len();let entry_j=(post_j+3)%4;
    let (rfirst,tend)=super::shared_step::SCHEDULE_SUPPORTS[block];let (lo,hi)=super::metadata_entry_head5::A_SUPPORTS[block];let previous=circ.q797_a_support.replace((lo,hi));
    // Iteration is not read until cycle exit. Borrow it, restore it, then exclude
    // it from that exit's lender list. This funds legacy max-control predicates.
    let pool:Vec<_>=helpers.iter().chain(std::iter::once(iteration)).map(QReg::borrowed_alias).collect();let sign=&pool[0];let dirty=&pool[1..];
    old::decode_birth(circ,rank,a,c,sm,p1,p2,w2,&pool,entry_j);
    old::loan(circ,rank,a,w1,sign,dirty);circ.ccx(p1,p2,sign);
    super::q798_handoffs::move_t10(circ,rank,a,p1,p2,w1,w2,dirty);
    super::metadata_arithmetic5_encoded::phase10_with_support(circ,rank,a,c,p1,p2,sign,w1,w2,dirty,tend);
    super::q798_handoffs::move_t10(circ,rank,a,p1,p2,w1,w2,dirty);circ.ccx(p1,p2,sign);old::loan(circ,rank,a,w1,sign,dirty);
    super::metadata_rotation5::rotate(circ,rank,a,p1,p2,w2,&pool,false);
    let p01=vec![(p1,false),(p2,true)];let empty=ceq(rank,c,0,&p01);let mut nonempty=vec![p01.clone()];nonempty.extend(empty.clone());
    moves::adjacent_a_terms(circ,rank,a,w2,1,2,&nonempty,&pool);
    super::metadata_remainder015_funded::signless(circ,rank,a,c,sm,p1,p2,w1,w2,&pool,entry_j,259-rfirst);
    // C0's coefficient-head loan is safe during arithmetic only. Afterwards
    // all R01 cargo can share the same gap before the counter advances C.
    moves::across_a_terms(circ,rank,a,w1,0,w2,2,&empty,true,&pool);
    old::loan(circ,rank,a,w1,sign,dirty);circ.ccx(p1,p2,sign);
    super::metadata_remainder5_phased::phase00_with_support(circ,rank,a,c,sm,p1,p2,sign,w1,w2,dirty,entry_j,259-rfirst);
    super::metadata_rotation5::rotate(circ,rank,a,p1,p2,w2,dirty,true);
    super::metadata_terminal5::emit(circ,rank,a,c,sm,dirty,post_j==0,false);
    super::metadata_phase_counter5::emit(circ,rank,a,c,sm,p1,p2,dirty,entry_j);
    let c1_t10=ceq(rank,c,1,&[(p1,true),(p2,false)]);
    moves::adjacent_a_terms(circ,rank,a,w1,2,3,&c1_t10,dirty);moves::flip_a_terms(circ,rank,a,w1,2,&c1_t10,dirty);
    super::metadata_entry_boundary5::entry_with_support(circ,rank,a,c,sm,p1,p2,sign,w1,w2,dirty,post_j,lo,hi);
    newborn(circ,rank,a,c,sm,p2,sign,w1,w2,dirty,post_j);
    // Continuing R00 returns its moving gap to the phase00 boundary address.
    let mut p00=vec![vec![(p1,false),(p2,false)]];let mut terminal=p00[0].clone();terminal.extend((0..5).map(|i|(&rank[i],29>>i&1!=0)));terminal.extend(a.iter().map(|q|(q,true)));p00.push(terminal);
    moves::adjacent_a_terms(circ,rank,a,w2,1,2,&p00,dirty);
    // Distinguish the newly empty C0 ascent from the fully assembled Q256.
    // W2[258] is outside every A-indexed gather used here (maximum index257).
    let mut empty=ceq(rank,c,0,&p01);
    if post_j==0 {let mut full=p01.clone();full.extend(rank.iter().chain(a).chain(c).chain(sm).map(|q|(q,false)));full.push((&w2[258],true));empty.push(full);}
    moves::across_a_terms(circ,rank,a,w2,1,w1,0,&empty,true,dirty);
    let mut nonempty=vec![p01.clone()];nonempty.extend(empty.clone());
    let zero=szero(rank,c,sm,post_j);let to_t10=product(&nonempty,&zero);let mut continuing=nonempty.clone();continuing.extend(to_t10.clone());
    moves::adjacent_a_terms(circ,rank,a,w2,2,0,&continuing,dirty);
    let c1=ceq(rank,c,1,&[]);let to_c1=product(&to_t10,&c1);let mut to_head=to_t10;to_head.extend(to_c1.clone());
    moves::across_a_terms(circ,rank,a,w2,2,w1,3,&to_c1,false,dirty);
    moves::across_a_terms(circ,rank,a,w2,2,w1,2,&to_head,true,dirty);
    old::encode_birth(circ,rank,a,c,sm,p1,p2,sign,dirty,post_j);old::hide_birth(circ,rank,a,c,sm,p1,p2,dirty,post_j);
    super::q798_sign_erase::emit(circ,rank,a,c,sm,p1,p2,sign,w1,w2,dirty,post_j,(tend+1).min(258));old::hide_birth(circ,rank,a,c,sm,p1,p2,dirty,post_j);old::loan(circ,rank,a,w1,sign,dirty);
    if post_j==0{super::metadata_exit_boundary5::exit_phase_cargo(circ,rank,a,c,sm,p1,p2,iteration,w1,w2,helpers,lo,hi);}
    old::phase_flips(circ,rank,a,c,sm,p1,p2,w1,&w2[258],&pool,post_j);
    let mut tail=circ.b.ops.split_off(start);super::shared_optimize::cancel_nct(&mut tail,2048,8);super::shared_optimize::cancel_nct_live(&mut tail,2048);circ.b.ops.extend(tail);circ.q797_a_support=previous;
}
