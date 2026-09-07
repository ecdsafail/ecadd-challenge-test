use super::*;

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
fn compact_adder_exact_values_phase_carry_and_restoration() {
 let mut lanes=0;
 for n in [1usize,2,3,4,5,8,16,32,40,64,128,256] {
  for conditioned in [false,true] {
   let mut b=Builder::new();let a=b.alloc_qubits(n);let v=b.alloc_qubits(n);
   let conditions=if conditioned{b.alloc_bits(2)}else{vec![]};
   for &c in &conditions {b.push_condition(c);}
   let cout=compact_add(&mut b,&a,&v);
   for _ in &conditions {b.pop_condition();}
   let ops=b.take_ops();let(nq,nb,_,_)=analyze_ops(ops.iter());assert_eq!(nq as usize,2*n+2);
   let(ar,vr)=(reg(&a),reg(&v));let batches=if n<=5 {(1usize<<(2*n)).div_ceil(64)}else{64};
   for batch in 0..batches {
    let mut fixed=Fixed(170);let mut sim=Simulator::new(nq as usize,nb as usize,&mut fixed);
    let mut rng=Shake256::default();rng.update(b"compact-adder-independent");rng.update(&(batch as u64).to_le_bytes());let mut rng=rng.finalize_xof();
    let(mut av,mut vv)=([U256::ZERO;64],[U256::ZERO;64]);let mut expected_carry=0;
    let cond=if conditioned{0xaaaaaaaaaaaaaaaa&0xcccccccccccccccc}else{u64::MAX};
    if conditioned{*sim.bit_mut(conditions[0])=0xaaaaaaaaaaaaaaaa;*sim.bit_mut(conditions[1])=0xcccccccccccccccc;}
    for lane in 0..64 {
     let k=batch*64+lane;
     if n<=5 {av[lane]=U256::from(k)&mask(n);vv[lane]=U256::from(k>>n)&mask(n);}
     else {let mut bytes=[0;32];rng.read(&mut bytes);av[lane]=U256::from_le_bytes(bytes)&mask(n);rng.read(&mut bytes);vv[lane]=U256::from_le_bytes(bytes)&mask(n);
      if batch<4 {vv[lane]=match batch{0=>av[lane],1=>mask(n)-av[lane],2=>U256::ZERO,_=>mask(n)};}}
     sim.set_register(&ar,av[lane],lane);sim.set_register(&vr,vv[lane],lane);
     expected_carry|=((av[lane]>mask(n)-vv[lane]) as u64)<<lane;
    }
    sim.apply_iter(ops.iter().filter(|op|op.kind!=OperationType::R));
    assert_eq!(sim.phase,0);assert_eq!(sim.qubit(cout),expected_carry&cond);
    assert_eq!(sim.qubit(QubitId((2*n+1) as u64)),0,"workspace");
    for lane in 0..64{assert_eq!(sim.get_register(&ar,lane),av[lane]);
     let want=if cond>>lane&1==0 {vv[lane]}else{av[lane].wrapping_add(vv[lane])&mask(n)};
     assert_eq!(sim.get_register(&vr,lane),want,"n={n} conditioned={conditioned} batch={batch}");
    }
    lanes+=64;
   }
  }
 }
 eprintln!("compact adder PASS {lanes} lanes, exact sum/carry, restored addend, zero phase, one scratch");
}

#[test]
fn compact_adder_carry_interfaces() {
 let mut lanes=0;
 for n in [1usize,2,3,4,5,8,16,32,40,64,128,256] {
  for conditioned in [false,true] {for incoming in [false,true] {for wrapped in [false,true] {
   let mut b=Builder::new();let a=b.alloc_qubits(n);let v=b.alloc_qubits(n);
   let conditions=if conditioned{b.alloc_bits(2)}else{vec![]};
   for &c in &conditions {b.push_condition(c);}
   let cin=incoming.then(||b.alloc_qubit());let cout=(!wrapped).then(||b.alloc_qubit());
   compact_add_with(&mut b,&a,&v,cin,cout);
   for _ in &conditions {b.pop_condition();}
   let ops=b.take_ops();let(nq,nb,_,_)=analyze_ops(ops.iter());assert!(nq as usize<=2*n+2);
   let(ar,vr)=(reg(&a),reg(&v));let batches=if n<=5 {(1usize<<(2*n)).div_ceil(64)}else{64};
   for batch in 0..batches {
    let mut fixed=Fixed(170);let mut sim=Simulator::new(nq as usize,nb as usize,&mut fixed);
    let mut rng=Shake256::default();rng.update(b"compact-adder-independent");rng.update(&(batch as u64).to_le_bytes());let mut rng=rng.finalize_xof();
    let(mut av,mut vv)=([U256::ZERO;64],[U256::ZERO;64]);let mut expected_carry=0;
    let cond=if conditioned{0xaaaaaaaaaaaaaaaa&0xcccccccccccccccc}else{u64::MAX};
    if conditioned{*sim.bit_mut(conditions[0])=0xaaaaaaaaaaaaaaaa;*sim.bit_mut(conditions[1])=0xcccccccccccccccc;}
    for lane in 0..64 {
     let k=batch*64+lane;
     if n<=5 {av[lane]=U256::from(k)&mask(n);vv[lane]=U256::from(k>>n)&mask(n);}
     else {let mut bytes=[0;32];rng.read(&mut bytes);av[lane]=U256::from_le_bytes(bytes)&mask(n);rng.read(&mut bytes);vv[lane]=U256::from_le_bytes(bytes)&mask(n);
      if batch<4 {vv[lane]=match batch{0=>av[lane],1=>mask(n)-av[lane],2=>U256::ZERO,_=>mask(n)};}}
     sim.set_register(&ar,av[lane],lane);sim.set_register(&vr,vv[lane],lane);
     expected_carry|=((av[lane]>mask(n)-vv[lane] || (incoming && lane%2==1 && av[lane]==mask(n)-vv[lane])) as u64)<<lane;
    }
    if let Some(cin)=cin {*sim.qubit_mut(cin)=0xaaaaaaaaaaaaaaaa;}
    sim.apply_iter(ops.iter().filter(|op|op.kind!=OperationType::R));
    assert_eq!(sim.phase,0);if let Some(cout)=cout {assert_eq!(sim.qubit(cout),expected_carry&cond);}
    if let Some(cin)=cin {assert_eq!(sim.qubit(cin),0xaaaaaaaaaaaaaaaa);}
    for q in 2*n..nq as usize {let q=QubitId(q as u64);if Some(q)!=cin && Some(q)!=cout {assert_eq!(sim.qubit(q),0,"workspace");}}
    for lane in 0..64{assert_eq!(sim.get_register(&ar,lane),av[lane]);
     let want=if cond>>lane&1==0 {vv[lane]}else{av[lane].wrapping_add(vv[lane]).wrapping_add(U256::from(u64::from(incoming && lane%2==1)))&mask(n)};
     assert_eq!(sim.get_register(&vr,lane),want,"n={n} conditioned={conditioned} batch={batch}");
    }
    lanes+=64;
   }
  }}}
 }
 eprintln!("compact adder interfaces PASS {lanes} lanes, exact sum/carry, restored addend, zero phase, one scratch");
}
