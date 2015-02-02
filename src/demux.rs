use logic_gates::{AndGate, NotGate, AndGateVec};
use sim::{NodeIndex, NodeCreator, STANDARD_DELAY};

/// Choose between two bits
pub struct BitDemux {
    pub input: NodeIndex,
    pub select: NodeIndex,
    pub output_a: NodeIndex,
    pub output_b: NodeIndex,
    pub enable: NodeIndex,
}

impl BitDemux {
    pub fn new(creator: &mut NodeCreator) -> BitDemux {
        let ander_a = AndGate::new(creator);
        let not_select = NotGate::new(creator);
        let ander_b = AndGate::new(creator);
        let enabler_a = AndGate::new(creator);
        let enabler_b = AndGate::new(creator);
        
        creator.link(not_select.output, ander_a.a, STANDARD_DELAY);
        creator.link(not_select.input, ander_b.a, STANDARD_DELAY);
        creator.link(ander_a.b, ander_b.b, STANDARD_DELAY);
        
        creator.link(enabler_a.a, enabler_b.a, STANDARD_DELAY);
        creator.link(enabler_a.b, ander_a.output, STANDARD_DELAY);
        creator.link(enabler_b.b, ander_b.output, STANDARD_DELAY);
        
        BitDemux {
            input: ander_a.b,
            select: not_select.input,
            output_a: enabler_a.output,
            output_b: enabler_b.output,
            enable: enabler_a.a,
        }
    }
}

// Choose between two words
pub struct Demux {
    pub input: Vec<NodeIndex>,
    pub select: NodeIndex,
    pub output_a: Vec<NodeIndex>,
    pub output_b: Vec<NodeIndex>,
    pub enable: NodeIndex,
}


impl Demux {
    pub fn new(word_bits: usize, creator: &mut NodeCreator) -> Demux {
        let bits : Vec<BitDemux> = range(0, word_bits).map(|_| { BitDemux::new(creator) } ).collect();
        
        for bit in (&bits[1..]).iter() {
            creator.link(bits[0].select, bit.select, STANDARD_DELAY);
            creator.link(bits[0].enable, bit.enable, STANDARD_DELAY);
        }
        
        Demux {
            input: bits.iter().map(|bit| { bit.input }).collect(),
            select: bits[0].select,
            enable: bits[0].enable,
            output_a: bits.iter().map(|bit| { bit.output_a }).collect(),
            output_b: bits.iter().map(|bit| { bit.output_b }).collect(),
        }
    }
}

// Choose between N words
pub struct DemuxN {
    pub input: Vec<NodeIndex>,
    pub select: Vec<NodeIndex>,
    pub outputs: Vec<Vec<NodeIndex>>,
    pub enable: NodeIndex,
}

impl DemuxN {
    pub fn new(word_bits: usize, word_count: usize, creator: &mut NodeCreator) -> DemuxN {
        assert!(word_count>0);
        
        if word_count == 1 {
            let ands = AndGateVec::new(word_bits, creator);
            creator.link_one_to_many(ands.b[0], &ands.b[], STANDARD_DELAY);
            DemuxN {
                input: ands.a,
                outputs: [ands.output].to_vec(),
                select: Vec::new(),
                enable: ands.b[0]
            }
        } else {
            let mut lower_size = 1;
            while lower_size*2 < word_count {
                lower_size *= 2;
            }
            
            println!("Making child demuxen: {} {}", lower_size, word_count - lower_size);
            let mut lower = DemuxN::new(word_bits, lower_size, creator);
            let mut upper = DemuxN::new(word_bits, word_count - lower_size, creator);
            
            creator.multilink(&lower.input[], &upper.input[], STANDARD_DELAY);
            
            let mut outputs = lower.outputs;
            outputs.append(&mut upper.outputs);
            assert_eq!(outputs.len(), word_count);
            
            let lower_enabler = AndGate::new(creator);
            let upper_enabler = AndGate::new(creator);
            let lower_select_gen = NotGate::new(creator);
            let upper_select = lower_select_gen.input;
            let lower_select = lower_select_gen.output;
            let enable = lower_enabler.b;
            creator.link(enable, upper_enabler.b, STANDARD_DELAY); // The enablers share one input -- the one enabling this whole thing
            creator.link(lower_enabler.a, lower_select, STANDARD_DELAY);
            creator.link(upper_enabler.a, upper_select, STANDARD_DELAY);
            creator.link(lower_enabler.output, lower.enable, STANDARD_DELAY);
            creator.link(upper_enabler.output, upper.enable, STANDARD_DELAY);
            
            let mut select = lower.select.clone();
            creator.multilink(&lower.select[..upper.select.len()], &upper.select[], STANDARD_DELAY);
            select.push(upper_select);
            
            
            DemuxN {
                input: lower.input,
                select: select,
                outputs: outputs,
                enable: enable,
            }
        }
    }
}


#[cfg(test)]
mod test {
    use truth_table::check_truth_table;
    use sim::{NodeCreator};
    use super::{BitDemux, Demux, DemuxN};
    
    #[test]
    fn test_bit_demux() {
        check_truth_table(|creator: &mut NodeCreator| {
            let demux = BitDemux::new(creator);
            
            ([demux.enable, demux.select, demux.input].to_vec(), [demux.output_a, demux.output_b].to_vec())
        }, &[
            (&[1, 0, 1], &[1, 0]),
            (&[1, 0, 0], &[0, 0]),
            (&[0, 0, 1], &[0, 0]),
            (&[0, 1, 1], &[0, 0]),
            (&[1, 1, 1], &[0, 1]),
        ]);
    }
    
    #[test]
    fn test_word_demux() {
        check_truth_table(|creator: &mut NodeCreator| {
            let demux = Demux::new(4, creator);
            
            let mut all_outputs = Vec::new();
            all_outputs.append(&mut demux.output_a.clone());
            all_outputs.append(&mut demux.output_b.clone());
            
            let mut all_inputs = Vec::new();
            all_inputs.push(demux.enable);
            all_inputs.push(demux.select);
            all_inputs.append(&mut demux.input.clone());
            
            (all_inputs, all_outputs)
        }, &[
            (&[1, 0, 1,0,0,1], &[1,0,0,1, 0,0,0,0]),
            (&[1, 1, 1,0,0,1], &[0,0,0,0, 1,0,0,1]),
            (&[0, 1, 1,0,0,1], &[0,0,0,0, 0,0,0,0]),
        ]);
    }
    
    #[test]
    fn test_demuxn() {
        check_truth_table(|creator: &mut NodeCreator| {
            let demux = DemuxN::new(4, 3, creator);
            
            let mut all_outputs = Vec::new();
            for output in demux.outputs.iter() {
                all_outputs.append(&mut output.clone());
            }
            
            let mut all_inputs = Vec::new();
            all_inputs.push(demux.enable);
            all_inputs.append(&mut demux.select.clone());
            all_inputs.append(&mut demux.input.clone());
            
            (all_inputs, all_outputs)
        }, &[
            (&[1, 0,0, 1,0,0,1], &[1,0,0,1, 0,0,0,0, 0,0,0,0]),
            (&[1, 0,1, 1,0,0,1], &[0,0,0,0, 0,0,0,0, 1,0,0,1]),
            (&[0, 0,1, 1,0,0,1], &[0,0,0,0, 0,0,0,0, 0,0,0,0]),
        ]);
    }
    
}
