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
fn blocked_fold_values_phase_and_clean_releases() {
    let mut lanes = 0;
    for w in [3usize,4,5,8,9,16,49,53,64,80,96,128,256] {
        for block in [2usize,8,16] {
            for conditioned in [false,true] {
                let mut b=Builder::new();
                let controls=b.alloc_qubits(4);let a=b.alloc_qubits(w);
                let conditions=if conditioned {b.alloc_bits(2)} else {vec![]};
                for &c in &conditions {b.push_condition(c);}
                selected(&mut b,&a,f(),controls[0],Some(controls[1]),controls[2],controls[3],block);
                for _ in &conditions {b.pop_condition();}
                assert_eq!(b.active_qubits() as usize,w+4);
                let ops=b.take_ops();for op in &ops {op.validate();}
                let(nq,nb,_,_)=analyze_ops(ops.iter());
                let ar=reg(&a);
                let batches=if w<=9 {(1usize<<(w+4)).div_ceil(64)} else {40};
                for batch in 0..batches {
                    let mut shake=Shake256::default();shake.update(b"blocked-fold-independent-20260907");
                    shake.update(&(w as u64).to_le_bytes());shake.update(&(batch as u64).to_le_bytes());
                    let mut rng=shake.finalize_xof();
                    let mut inputs=[U256::ZERO;64];let mut cm=[0u64;4];
                    for lane in 0..64 {
                        let k=batch*64+lane;
                        for j in 0..4 {cm[j]|=(((k>>j)&1) as u64)<<lane;}
                        let mut bytes=[0;32];rng.read(&mut bytes);
                        inputs[lane]=if w<=9 {U256::from(k>>4)&mask(w)} else {U256::from_le_bytes(bytes)&mask(w)};
                        if w>9 && batch<8 {
                            inputs[lane]=match batch {
                                0=>U256::ZERO,1=>mask(w),2=>f()&mask(w),3=>U256::ZERO.wrapping_sub(f())&mask(w),
                                4=>mask(w.min(33)),5=>mask(w)^mask(w.min(33)),
                                6=>mask(w.min(block+1)),_=>mask(w)^mask(w.min(block+1)),
                            };
                        }
                    }
                    for measurement in [0u8,255,170,93] {
                        let mut fixed=Fixed(measurement);
                        let mut random=Shake256::default();random.update(b"blocked-fold-hmr");random.update(&(batch as u64).to_le_bytes());
                        let mut random=random.finalize_xof();
                        let reader:&mut dyn XofReader=if measurement==93 {&mut random}else{&mut fixed};
                        let mut reader=Reader(reader);let mut sim=Simulator::new(nq as usize,nb as usize,&mut reader);
                        for j in 0..4 {*sim.qubit_mut(controls[j])=cm[j];}
                        let cond=if conditioned {0xaaaaaaaaaaaaaaaa & 0xcccccccccccccccc} else {u64::MAX};
                        if conditioned {*sim.bit_mut(conditions[0])=0xaaaaaaaaaaaaaaaa;*sim.bit_mut(conditions[1])=0xcccccccccccccccc;}
                        for lane in 0..64 {sim.set_register(&ar,inputs[lane],lane);}
                        // One apply_iter call preserves nested condition stacks. Removing R
                        // in this independent component test prevents resets from hiding
                        // dirty scratch: every released wire must be reusable as-is.
                        sim.apply_iter(ops.iter().filter(|op| op.kind!=OperationType::R));
                        assert_eq!(sim.phase,0,"phase w={w} block={block} batch={batch} hmr={measurement}");
                        for j in 0..4 {assert_eq!(sim.qubit(controls[j]),cm[j]);}
                        for q in w+4..nq as usize {assert_eq!(sim.qubit(QubitId(q as u64)),0,"scratch");}
                        for lane in 0..64 {
                            let mut operand=U256::ZERO;
                            if cm[0]>>lane&1==1 {operand^=f();}
                            if cm[1]>>lane&1==1 {operand^=f()<<1;}
                            if cm[2]>>lane&1==1 {operand^=U256::ZERO.wrapping_sub(f());}
                            let cin=U256::from(cm[3]>>lane&1);
                            let high=(inputs[lane]>>1)+(operand>>1)+cin;
                            let want=if cond>>lane&1==0 {inputs[lane]} else {
                                ((high<<1)|((inputs[lane]^operand)&U256::from(1)))&mask(w)
                            };
                            assert_eq!(sim.get_register(&ar,lane),want,"value w={w} block={block} batch={batch} lane={lane}");
                        }
                        lanes+=64;
                    }
                }
            }
        }
    }
    eprintln!("blocked fold PASS {lanes} lanes: values, all four preserved controls, phase, reset-free scratch reuse and nested conditions");
}

#[test]
fn blocked_fold_resource_frontier() {
    for w in [53usize,64,80,96,128,256] {
        for block in [0usize,4,8,12,16] {
            let mut b=Builder::new();let c=b.alloc_qubits(4);let a=b.alloc_qubits(w);
            if block==0 {fold_selected(&mut b,&a,f(),c[0],Some(c[1]),c[2],c[3]);}
            else {selected(&mut b,&a,f(),c[0],Some(c[1]),c[2],c[3],block);}
            let ops=b.take_ops();let(q,_,_,_)=analyze_ops(ops.iter());
            let (mut u,mut conditional,mut depth)=(0,0,0);
            for op in &ops {
                if op.kind==OperationType::PushCondition {depth+=1;}
                if op.kind==OperationType::PopCondition {depth-=1;}
                if matches!(op.kind,OperationType::CCX|OperationType::CCZ) {
                if depth==0 && op.c_condition.0==u64::MAX {u+=1;}else{conditional+=1;}
            }}
            eprintln!("FOLD_RES w={w} block={block} Q={q} U={u} C={conditional}");
        }
    }
}

#[test]
fn blocked_constant_values_phase_and_reset_free_scratch() {
    let mut lanes=0;
    for w in [4usize,8,16,33,53,64,80,96,128,256] {
        for c in [U256::ZERO,U256::from(1),U256::from(2),f(),f()-U256::from(1),U256::ZERO.wrapping_sub(f()),U256::MAX] {
            let mut b=Builder::new();let control=b.alloc_qubit();let a=b.alloc_qubits(w);
            constant(&mut b,&a,c,control,8);
            assert_eq!(b.active_qubits() as usize,w+1);
            let ops=b.take_ops();let(nq,nb,_,_)=analyze_ops(ops.iter());let ar=reg(&a);
            for batch in 0..40 {
                for measurement in [0u8,255,170] {
                    let mut fixed=Fixed(measurement);let mut sim=Simulator::new(nq as usize,nb as usize,&mut fixed);
                    *sim.qubit_mut(control)=0xaaaaaaaaaaaaaaaa;
                    let mut rng=Shake256::default();rng.update(b"blocked-constant-independent");rng.update(&(batch as u64).to_le_bytes());let mut rng=rng.finalize_xof();
                    let mut input=[U256::ZERO;64];
                    for lane in 0..64 {
                        let mut bytes=[0;32];rng.read(&mut bytes);
                        input[lane]=if w<=8 {U256::from(batch*32+lane/2)&mask(w)} else if batch==0 {U256::ZERO} else if batch==1 {mask(w)} else if batch==2 {U256::ZERO.wrapping_sub(c)&mask(w)} else {U256::from_le_bytes(bytes)&mask(w)};
                        sim.set_register(&ar,input[lane],lane);
                    }
                    sim.apply_iter(ops.iter().filter(|op|op.kind!=OperationType::R));
                    assert_eq!(sim.phase,0,"constant phase w={w} c={c} batch={batch}");
                    assert_eq!(sim.qubit(control),0xaaaaaaaaaaaaaaaa);
                    for lane in 0..64 {
                        let wanted=if lane%2==0 {input[lane]} else {input[lane].wrapping_add(c)&mask(w)};
                        assert_eq!(sim.get_register(&ar,lane),wanted,"constant value w={w} c={c} batch={batch}");
                    }
                    for q in w+1..nq as usize {assert_eq!(sim.qubit(QubitId(q as u64)),0,"constant scratch");}
                    lanes+=64;
                }
            }
        }
    }
    eprintln!("blocked constant PASS {lanes} lanes");
}
