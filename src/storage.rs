
use sim::{LineState, NodeIndex, NodeCreator, Element, NodeCollection, PropogationDelay, STANDARD_DELAY};

use logic_gates::{Nand, AndGate};

use arena::Arena;

#[derive(Debug)]
pub struct NotSRLatch {
    pub not_s: NodeIndex,
    pub not_r: NodeIndex,
    pub q: NodeIndex,
    pub not_q: NodeIndex,
}

impl NotSRLatch {
    pub fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> NotSRLatch {
        let top = arena.alloc(|| { Nand::new(creator) });
        let bottom = arena.alloc(|| { Nand::new(creator) });
        
        creator.add_element(top);
        creator.add_element(bottom);
        
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
    pub fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> DFlipFlop {
        let top = NotSRLatch::new(creator, arena);
        let bottom = NotSRLatch::new(creator, arena);
        let output = NotSRLatch::new(creator, arena);
        let ander = AndGate::new(creator, arena);
        
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
    pub fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena, bit_count: usize) -> Register {
        let bits : Vec<DFlipFlop> = range(0, bit_count).map(|_| { DFlipFlop::new(creator, arena) }).collect();
        
        let clock = bits[0].clock;
        for bit in bits.slice_from(1).iter() {
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
