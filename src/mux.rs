use sim::{NodeIndex, NodeCreator, STANDARD_DELAY};
use logic_gates::{AndGate, OrGate, NotGate};

pub struct BitMux {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub select: NodeIndex,
    pub output: NodeIndex,
}

impl BitMux {
    pub fn new(creator: &mut NodeCreator) -> BitMux {
        let not_select = NotGate::new(creator);
        let a_masked = AndGate::new(creator);
        let b_masked = AndGate::new(creator);
        let output = OrGate::new(creator);
        
        creator.link(not_select.output, a_masked.a, STANDARD_DELAY);
        creator.link(not_select.input, b_masked.a, STANDARD_DELAY);
        creator.link(a_masked.output, output.a, STANDARD_DELAY);
        creator.link(b_masked.output, output.b, STANDARD_DELAY);
        
        BitMux {
            a: a_masked.b,
            b: b_masked.b,
            select: not_select.input,
            output: output.output,
        }
    }
}

/// Selects between two arbitrary-length inputs
pub struct Mux {
    pub a: Vec<NodeIndex>,
    pub b: Vec<NodeIndex>,
    pub output: Vec<NodeIndex>,
    pub select: NodeIndex,
}

impl Mux {
    pub fn new(word_bits: usize, creator: &mut NodeCreator) -> Mux {
        assert!(word_bits>0);
    
        let bits : Vec<BitMux> = range(0, word_bits).map(|_| { BitMux::new(creator) }).collect();
        
        for bit in (&bits[1..]).iter() {
            creator.link(bits[0].select, bit.select, STANDARD_DELAY);
        }
        
        Mux {
            a: bits.iter().map(|bit| { bit.a }).collect(),
            b: bits.iter().map(|bit| { bit.b }).collect(),
            output: bits.iter().map(|bit| { bit.output }).collect(),
            select: bits[0].select,
        }
    }
}
