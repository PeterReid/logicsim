
use sim::{LineState, NodeIndex, NodeCreator, NodeCollection, PropogationDelay, STANDARD_DELAY};

use logic_gates::{NandGate, AndGate};

use arena::Arena;

#[derive(Debug)]
pub struct NotSRLatch {
    pub not_s: NodeIndex,
    pub not_r: NodeIndex,
    pub q: NodeIndex,
    pub not_q: NodeIndex,
}

impl NotSRLatch {
    pub fn new(creator: &mut NodeCreator) -> NotSRLatch {
        let top = NandGate::new(creator);
        let bottom = NandGate::new(creator);
        
        creator.link(top.output, bottom.a, STANDARD_DELAY);
        creator.link(bottom.output, top.b, STANDARD_DELAY);
        
        NotSRLatch{
            not_s: top.a,
            not_r: bottom.b,
            q: top.output,
            not_q: bottom.output,
        }
    }
}


#[derive(Debug)]
pub struct DFlipFlop {
    pub clock: NodeIndex,
    pub data: NodeIndex,
    pub q: NodeIndex,
    pub not_q: NodeIndex,
}

impl DFlipFlop {
    pub fn new(creator: &mut NodeCreator) -> DFlipFlop {
        let top = NotSRLatch::new(creator);
        let bottom = NotSRLatch::new(creator);
        let output = NotSRLatch::new(creator);
        let ander = AndGate::new(creator);
        
        let clock = ander.a;
        
        creator.link(ander.b, top.not_q, STANDARD_DELAY);
        creator.link(ander.output, bottom.not_s, STANDARD_DELAY);
        let data = bottom.not_r;
        
        creator.link(bottom.q, output.not_r, STANDARD_DELAY);
        creator.link(clock, top.not_r, STANDARD_DELAY);
        creator.link(bottom.not_q, top.not_s, STANDARD_DELAY);
        creator.link(top.not_q, output.not_s, STANDARD_DELAY);
        
        DFlipFlop {
            clock: clock,
            data: data,
            q: output.q,
            not_q: output.not_q
        }
    }
}

pub struct Register {
    pub bits: Vec<DFlipFlop>,
    pub clock: NodeIndex,
}

impl Register {
    pub fn new(creator: &mut NodeCreator, bit_count: usize) -> Register {
        let bits : Vec<DFlipFlop> = range(0, bit_count).map(|_| { DFlipFlop::new(creator) }).collect();
        
        let clock = bits[0].clock;
        for bit in (&bits[1..]).iter() {
            creator.link(clock, bit.clock, STANDARD_DELAY);
        }
        
        Register {
            bits: bits,
            clock: clock,
        }
    }
    
    pub fn read_u64(&self, c: &NodeCollection) -> Option<u64> {
        let mut accum = 0u64;
        for (index, bit) in self.bits.iter().enumerate() {
            match bit.q.read(c) {
                LineState::Low => {},
                LineState::High => {
                    accum |= 1<<index;
                },
                _ => { return None; }
            }
        }
        
        return Some(accum);
    }
}
