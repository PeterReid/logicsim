use super::super::sim::{NodeIndex, NodeCreator, STANDARD_DELAY};
use super::super::mux::{MuxN};
use super::super::demux::{DemuxN};
use super::super::storage::Register;
use super::params::Params;

struct RegisterBank {
    write_selector: Vec<NodeIndex>,
    read_selector: Vec<NodeIndex>,
    input: Vec<NodeIndex>,
    output: Vec<NodeIndex>,
    write_clock: NodeIndex, // When this goes high, input gets stored.
}

impl RegisterBank {
    fn new(params: &Params, creator: &mut NodeCreator) -> RegisterBank {
        let register_count = 1 << params.log_register_count;
        let registers: Vec<Register> = range(0, register_count).map(|_| { Register::new(creator, params.word_bits) }).collect();
        
        // Wire all inputs together
        for register in (&registers[1..]).iter() {
            creator.multilink(&registers[0].inputs[], &register.inputs[], STANDARD_DELAY);
        }
        
        let output_chooser = MuxN::new(params.word_bits, register_count, creator);
        for (register, output_chooser_source) in registers.iter().zip(output_chooser.inputs.iter()) {
            creator.multilink(&register.outputs[], &output_chooser_source[], STANDARD_DELAY);
        }
        
        let demux = DemuxN::new(1, register_count, creator);
        assert_eq!(demux.outputs.len(), registers.len());
        for (register, demux_output) in registers.iter().zip(demux.outputs.iter()) {
            assert_eq!(demux_output.len(), 1);
            creator.link(demux_output[0], register.clock, STANDARD_DELAY);
        }
        
        RegisterBank {
            input: registers[0].inputs.clone(),
            write_selector: demux.select,
            read_selector: output_chooser.select,
            write_clock: demux.input[0],
            output: output_chooser.output,
        }
    }
}