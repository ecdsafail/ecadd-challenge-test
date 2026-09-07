use super::*;
use alloy_primitives::U256;
use crate::circuit::{analyze_ops,OperationType,QubitOrBit};
use crate::sim::Simulator;
use sha3::{digest::{ExtendableOutput,Update,XofReader},Shake256};

struct Fixed(u8);
impl XofReader for Fixed {fn read(&mut self,d:&mut[u8]) {d.fill(self.0);}}
struct Reader<'a>(&'a mut dyn XofReader);
impl XofReader for Reader<'_> {fn read(&mut self,d:&mut[u8]) {self.0.read(d);}}
fn reg(q:&[QubitId])->Vec<QubitOrBit> {q.iter().copied().map(QubitOrBit::Qubit).collect()}
fn mask(w:usize)->U256 {if w==256 {U256::MAX} else {(U256::from(1)<<w)-U256::from(1)}}



#[test]
fn compact_comparison_exact_phase_and_restoration() {
 let mut lanes=0;
 for n in [1usize,2,3,4,5,8,16,32,40,64,128,256] {
  for incoming in [false,true] {for conditioned in [false,true] {
   let mut b=Builder::new();let a=b.alloc_qubits(n);let v=b.alloc_qubits(n);
   let cin=incoming.then(||b.alloc_qubit());let conditions=if conditioned{b.alloc_bits(2)}else{vec![]};
   for &c in &conditions {b.push_condition(c);}
   cmp_lt_phase_compact(&mut b,&a,&v,cin);
   for _ in &conditions {b.pop_condition();}
   let ops=b.take_ops();let(nq,nb,_,_)=analyze_ops(ops.iter());
   assert!(nq as usize<=2*n+1);
   let(ar,vr)=(reg(&a),reg(&v));
   let batches=if n<=5 {(1usize<<(2*n+1)).div_ceil(64)}else{64};
   for batch in 0..batches {
    let mut fixed=Fixed(170);let mut sim=Simulator::new(nq as usize,nb as usize,&mut fixed);
    let mut rng=Shake256::default();rng.update(b"compact-compare-independent");rng.update(&(batch as u64).to_le_bytes());let mut rng=rng.finalize_xof();
    let(mut av,mut vv)=([U256::ZERO;64],[U256::ZERO;64]);let mut cm=0;let mut phase=0;
    let cond=if conditioned{0xaaaaaaaaaaaaaaaa&0xcccccccccccccccc}else{u64::MAX};
    if conditioned{*sim.bit_mut(conditions[0])=0xaaaaaaaaaaaaaaaa;*sim.bit_mut(conditions[1])=0xcccccccccccccccc;}
    for lane in 0..64 {
     let k=batch*64+lane;let c=incoming&&(k&1!=0);cm|=(c as u64)<<lane;
     if n<=5 {av[lane]=U256::from(k>>1)&mask(n);vv[lane]=U256::from(k>>(n+1))&mask(n);}
     else {let mut bytes=[0;32];rng.read(&mut bytes);av[lane]=U256::from_le_bytes(bytes)&mask(n);rng.read(&mut bytes);vv[lane]=U256::from_le_bytes(bytes)&mask(n);
      if batch<4 {vv[lane]=match batch{0=>av[lane],1=>av[lane].wrapping_add(U256::from(1))&mask(n),2=>U256::ZERO,_=>mask(n)};}}
     sim.set_register(&ar,av[lane],lane);sim.set_register(&vr,vv[lane],lane);
     phase|=((if c {av[lane]<=vv[lane]}else{av[lane]<vv[lane]}) as u64)<<lane;
    }
    if let Some(c)=cin{*sim.qubit_mut(c)=cm;}
    sim.apply_iter(ops.iter().filter(|op|op.kind!=OperationType::R));
    assert_eq!(sim.phase,phase&cond,"n={n} incoming={incoming} conditioned={conditioned} batch={batch}");
    for lane in 0..64{assert_eq!(sim.get_register(&ar,lane),av[lane]);assert_eq!(sim.get_register(&vr,lane),vv[lane]);}
    if let Some(c)=cin{assert_eq!(sim.qubit(c),cm);}else{assert_eq!(sim.qubit(QubitId((2*n) as u64)),0);}
    lanes+=64;
   }
  }}
 }
 eprintln!("compact compare PASS {lanes} lanes, exact phase and restored operands/borrow, <=1 scratch");
}
