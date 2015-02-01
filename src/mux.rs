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

pub struct MuxN {
    pub inputs: Vec<Vec<NodeIndex>>,
    pub output: Vec<NodeIndex>,
    pub select: Vec<NodeIndex>,
}

impl MuxN {
    pub fn new(word_bits: usize, word_count: usize, creator: &mut NodeCreator) -> MuxN {
        assert!(word_count >= 1);
        
        if word_count == 1 {
            let nodes : Vec<NodeIndex> = range(0, word_bits).map(|_| { creator.new_node() }).collect();
            MuxN {
                inputs: [nodes.clone()].to_vec(),
                output: nodes.clone(),
                select: Vec::new()
            }
        } else {
            let mut lower_size = 1;
            while lower_size*2 < word_count {
                lower_size *= 2;
            }
            
            let mut lower = MuxN::new(word_bits, lower_size, creator);
            let mut upper = MuxN::new(word_bits, word_count - lower_size, creator);
            let top_level_chooser = Mux::new(word_bits, creator);
            creator.multilink(&lower.output[], &top_level_chooser.a[], STANDARD_DELAY);
            creator.multilink(&upper.output[], &top_level_chooser.b[], STANDARD_DELAY);
            creator.multilink(&lower.select[..upper.select.len()], &upper.select[], STANDARD_DELAY);
            
            let mut select = lower.select;
            select.push(top_level_chooser.select);
            let mut inputs = lower.inputs;
            inputs.append(&mut upper.inputs);
            assert_eq!(inputs.len(), word_count);
            
            MuxN {
                inputs: inputs,
                output: top_level_chooser.output,
                select: select,
            }
        }
    }
}



#[cfg(test)]
mod test {
    use truth_table::check_truth_table;
    use sim::{NodeCreator};
    use super::{MuxN};
    
    #[test]
    fn test_muxn() {
        check_truth_table(|creator: &mut NodeCreator| {
            let mux = MuxN::new(4, 3, creator);
            
            let mut all_inputs = Vec::new();
            all_inputs.append(&mut mux.select.clone());
            for input in mux.inputs.iter() {
                all_inputs.append(&mut input.clone());
            }
            
            (all_inputs, mux.output.clone())
        }, &[
            (&[0,0,  1,1,1,1, 1,0,0,0, 0,1,0,1], &[1,1,1,1]),
            (&[1,0,  1,1,1,1, 1,0,0,0, 0,1,0,1], &[1,0,0,0]),
            (&[0,1,  1,1,1,1, 1,0,0,0, 0,1,0,1], &[0,1,0,1]),
        ]);
    }

}
