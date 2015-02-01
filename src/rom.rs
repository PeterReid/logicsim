use sim::{NodeIndex, NodeCreator, NodeCollection, Element, PropogationDelay, STANDARD_DELAY, LineState};
use pin::Pin;
use mux::MuxN;

use arena::Arena;

struct ConstantBitElem {
    node: NodeIndex,
    on: bool,
}

impl ConstantBitElem {
    fn new(on: bool, c: &mut NodeCreator) -> ConstantBitElem {
        ConstantBitElem {
            node: c.new_node(),
            on: on,
        }
    }
}

impl Element for ConstantBitElem {
    fn step(&self, c: &mut NodeCollection) {
        let state = if self.on { LineState::High } else { LineState::Low };
        self.node.write(state, c);
    }
    
    fn get_nodes(&self) -> Vec<NodeIndex> {
        let mut v = Vec::new();
        v.push(self.node);
        v
    }
}

pub struct ConstantBit {
    pub node: NodeIndex,
}

impl ConstantBit {
    pub fn new(on: bool, creator: &mut NodeCreator) -> ConstantBit {
        let elem = creator.arena.alloc(|| { ConstantBitElem::new(on, creator) });
        creator.add_element(elem);
        ConstantBit {
            node: elem.node
        }
    }
}


pub struct ConstantBits {
    pub bits: Vec<NodeIndex>
}

impl ConstantBits {
    pub fn make_bits(value: u64, bit_count: usize) -> Vec<bool> {
        assert!(bit_count <= 64);
        range(0, bit_count).map(|bit_index| {
            (value & (1 << bit_index)) != 0
        }).collect()
    }

    pub fn new(bits: &[bool], creator: &mut NodeCreator) -> ConstantBits {
        ConstantBits {
            bits: bits.iter().map(|bit_on| {
                ConstantBit::new(*bit_on, creator).node
            }).collect()
        }
    }
}



pub struct Rom {
    address: Vec<NodeIndex>,
    output: Vec<NodeIndex>,
}
impl Rom {
    pub fn new(content: &[&[bool]], creator: &mut NodeCreator) -> Rom {
        let word_count = content.len();
        assert!(word_count>0);
        let word_bits = content[0].len();
        
        let mux = MuxN::new(word_bits, word_count, creator);
        assert_eq!(content.len(), mux.inputs.len());
        for (content_word, mux_input_word) in content.iter().zip(mux.inputs.iter()) {
            let constant_generator = ConstantBits::new(*content_word, creator);
            creator.multilink(&constant_generator.bits[], &mux_input_word[], STANDARD_DELAY);
        }
        
        Rom {
            address: mux.select,
            output: mux.output,
        }
    }
}

#[cfg(test)]
mod test {
    use truth_table::check_truth_table;
    use sim::{NodeCreator};
    use super::{Rom, ConstantBits};
    
    #[test]
    fn test_rom() {
        check_truth_table(|creator: &mut NodeCreator| {
            let rom = Rom::new(&[
                &ConstantBits::make_bits(5, 8)[],
                &ConstantBits::make_bits(128, 8)[],
                &ConstantBits::make_bits(255, 8)[],
            ], creator);
            
            (rom.address.clone(), rom.output.clone())
        }, &[
            (&[0,0], &[1,0,1,0,0,0,0,0]),
            (&[1,0], &[0,0,0,0,0,0,0,1]),
            (&[0,1], &[1,1,1,1,1,1,1,1]),
        ]);
    }

}