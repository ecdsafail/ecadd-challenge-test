//! One variable-width adder rather than four separately guarded adders.
//! The clean-under-guard mask is loaned from the extracted quotient slot.
use crate::point_add::trailmix_port::circuit::{Circuit,QReg};
use super::length_recompute::mixed_mcx;
#[path="metadata_arithmetic5_programs.rs"] mod programs;
struct Range<'a>{rank:&'a[QReg],low:&'a[QReg],guard:&'a QReg,cache:&'a QReg,mask:&'a QReg,dirty:&'a[QReg],group:isize,threshold:usize}
impl Range<'_>{
    fn high(&self,circ:&mut Circuit,h:isize){if !(0..4).contains(&h){return;}for &(m,v)in programs::A_EQUAL[h as usize]{let mut cs=vec![(self.guard,true)];cs.extend((0..5).filter(|&i|m>>i&1!=0).map(|i|(&self.rank[i],v>>i&1!=0)));mixed_mcx(circ,&cs,self.cache,self.dirty);}}
    fn select(&mut self,circ:&mut Circuit,h:isize){if h!=self.group{self.high(circ,self.group);self.high(circ,h);self.group=h;}}
    fn equality(&mut self,circ:&mut Circuit,value:usize,extra:&[(&QReg,bool)],target:&QReg){if value>255{return;}self.select(circ,(value/64)as isize);let mut cs=vec![(self.guard,true),(self.cache,true)];cs.extend((0..6).map(|i|(&self.low[i],value>>i&1!=0)));cs.extend_from_slice(extra);mixed_mcx(circ,&cs,target,self.dirty);}
    // mask = [A >= threshold] on guard; threshold is clipped to 0..256.
    fn set(&mut self,circ:&mut Circuit,t:usize){let t=t.min(256);for value in self.threshold.min(t)..self.threshold.max(t){self.equality(circ,value,&[],self.mask);}self.threshold=t;}
}
pub(super) fn prefix(circ:&mut Circuit,rank:&[QReg],source:&[QReg],target:&[QReg],low:&[QReg],guard:&QReg,cache:&QReg,mask:&QReg,sign_out:Option<&QReg>,sign_control:Option<&QReg>,dirty:&[QReg],subtract:bool,n:usize){
    let start=circ.b.ops.len();let mut r=Range{rank,low,guard,cache,mask,dirty,group:-1,threshold:0};circ.cx(guard,mask);
    let mut cells:Vec<(usize,u8,Vec<&QReg>,&QReg)>=Vec::new();
    for i in 0..n{cells.push((i,2,vec![&source[i]],&target[i]));}cells.push((0,3,vec![&source[0]],&target[0]));
    for i in (1..n).rev(){if let Some(z)=sign_out{cells.push((i,1,vec![&source[i]],z));}if i+1<n{cells.push((i+1,2,vec![&source[i]],&source[i+1]));}}
    for i in 0..n{if i+1<n{cells.push((i+1,0,vec![&source[i],&target[i]],&source[i+1]));}if let Some(z)=sign_out{cells.push((i,1,vec![&source[i],&target[i]],z));}}
    for i in (1..n).rev(){cells.push((i,0,vec![&source[i]],&target[i]));cells.push((i,0,vec![&source[i-1],&target[i-1]],&source[i]));}
    for i in 1..n-1{cells.push((i+1,2,vec![&source[i]],&source[i+1]));}for i in 0..n{cells.push((i,2,vec![&source[i]],&target[i]));}
    for(i,tag,data,out)in cells{
        if tag==2{circ.cx(data[0],out);continue;}
        let mut extras:Vec<_>=data.iter().map(|&q|(q,true)).collect();if let Some(s)=sign_control{extras.push((s,false));}
        if tag==1{if i>=1{r.equality(circ,i-1,&extras,out);}continue;}
        let mut cs=vec![(guard,true)];if tag==0{r.set(circ,i.saturating_sub(1));cs.push((mask,true));}cs.extend(extras);mixed_mcx(circ,&cs,out,dirty);
    }
    r.set(circ,0);r.select(circ,-1);circ.cx(guard,mask);
    if subtract{circ.b.ops[start..].reverse();}
}
pub fn run(){
    use crate::{sim::Simulator,circuit::OperationType};use sha3::digest::XofReader;
    struct Fixed;impl XofReader for Fixed{fn read(&mut self,b:&mut[u8]){b.fill(0x69)}}
    fn rnd(s:&mut u64)->u64{*s^=*s<<13;*s^=*s>>7;*s^=*s<<17;*s}
    fn put(w:&mut[u64],q:&QReg,l:usize,v:bool){let b=1u64<<l;w[q.id()as usize]=(w[q.id()as usize]&!b)|if v{b}else{0};}
    let ts:Vec<_>=(0..4).flat_map(|a|(0..4).flat_map(move|c|(0..4).filter(move|&s|a+c+s<=4).map(move|s|[a,c,s]))).collect();
    for sub in [false,true]{let mut circ=Circuit::new();let rank=circ.alloc_qreg_bits("rank",5);let low=circ.alloc_qreg_bits("low",6);let g=circ.alloc_qreg("guard");let cache=circ.alloc_qreg("cache");let mask=circ.alloc_qreg("mask");let sign=circ.alloc_qreg("sign");let a=circ.alloc_qreg_bits("source",259);let b=circ.alloc_qreg_bits("target",259);let dirty=circ.alloc_qreg_bits("dirty",24);let n=circ.b.next_qubit;
        prefix(&mut circ,&rank,&a,&b,&low,&g,&cache,&mask,if sub{None}else{Some(&sign)},if sub{Some(&sign)}else{None},&dirty,sub,259);let ops=circ.into_builder().ops;
        for batch in 0..128{let mut seed=79955^batch;let mut before:Vec<_>=(0..n).map(|_|rnd(&mut seed)).collect();let mut after=before.clone();for l in 0..64{let k=batch as usize*64+l;let r=k&31;let lo=k>>5&63;let on=k>>11&1!=0;let width=64*ts[r][0]+lo+2;
            for w in [&mut before,&mut after]{for i in 0..5{put(w,&rank[i],l,r>>i&1!=0);}for i in 0..6{put(w,&low[i],l,lo>>i&1!=0);}put(w,&g,l,on);if on{put(w,&cache,l,false);put(w,&mask,l,false);}}
            let sv=before[sign.id()as usize]>>l&1!=0;if on&&(!sub||!sv){let mut carry=false;for i in 0..width{let av=before[a[i].id()as usize]>>l&1!=0;let bv=before[b[i].id()as usize]>>l&1!=0;put(&mut after,&b[i],l,av^bv^carry);carry=if sub{(!bv&&(av||carry))||(av&&carry)}else{(av&&bv)||(av&&carry)||(bv&&carry)};}if !sub{put(&mut after,&sign,l,sv^carry);}}
        }let mut f=Fixed;let mut sim=Simulator::new(n as usize,0,&mut f);sim.qubits.copy_from_slice(&before);sim.apply_iter(ops.iter());if sim.qubits!=after{let diffs:Vec<_>=sim.qubits.iter().zip(&after).enumerate().filter(|(_, (x,y))|x!=y).map(|(i,(x,y))|(i,format!("{:016x}",x^y))).collect();panic!("PREFIX_UNIT sub={sub} batch={batch} diffs={diffs:?}");}assert_eq!(sim.phase,0);sim.apply_iter(ops.iter().rev());assert_eq!(sim.qubits,before);}
        eprintln!("PREFIX_UNIT_PASS subtract={sub} cases=8192 ops={} T={}",ops.len(),ops.iter().filter(|o|o.kind==OperationType::CCX).count());
    }
}
