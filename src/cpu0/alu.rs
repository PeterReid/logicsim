use super::super::sim::{NodeIndex, NodeCreator, STANDARD_DELAY};
use super::super::adder::RippleCarryAdder;
use super::super::logic_gates::{AndGateVec, XorGateVec};
use super::super::rom::Rom;
use super::super::mux::Mux;
use super::params::Params;

struct Alu {
    a: Vec<NodeIndex>,
    b: Vec<NodeIndex>,
    mode: Vec<NodeIndex>,
    output: Vec<NodeIndex>,
}

/// Functions are
/// - 0: 0 (ZN)
/// - 1: identity (Z)
/// - 2: add (no flags)
/// - 3: subtract (IC)
/// - 4: increment (ZC)
/// - 5: decrement (ZI)
/// - 6: and (N)
/// - 7: unused
///
/// Control lines are:
/// - Z: Does B get replaced by 0? 
/// - I: Invert B, post-zero/pre-adder?
/// - C: Carry-in set?
/// - N: Do we select from the ANDer or the ADDer or the NOTer?
///
//     --------------------
//     |                 |
///    A      B          |
///    |      |          |
///    |     (Z)         |
///    |      |--------| |
///    |     (I)       And
///    /-------\        |  
///    | Adder |--- C   |  
///    \-------/        |  
///        |     N      |
///        ----Select-----
///              |
///            Output
impl Alu {
    pub fn new(params: &Params, creator: &mut NodeCreator) -> Alu {
        let control_rom = Rom::new(&[
            // nonzro invert carry  and
            &[ false, false, false, true  ], // zero
            &[ false, false, false, false ], // identity
            &[ false, false, true,  false ], // increment
            &[ false, true,  false, false ], // decrement
            &[ true,  false, false, false ], // add
            &[ true,  true,  true,  false ], // subtract
            &[ true,  false, false, true  ], // and
        ], creator);
        
        let keep_nonzero = control_rom.output[0];
        let do_invert = control_rom.output[1];
        let carry_in_set = control_rom.output[2];
        let select_and = control_rom.output[3];
        
        //
        // Build the adder-using branch
        //
        
        // Possibly mask B to 0
        let masked_b = AndGateVec::new(params.word_bits, creator);
        creator.link_one_to_many(keep_nonzero, &masked_b.b[], STANDARD_DELAY);
        let b_raw_input = &masked_b.a;
        
        // Possibly invert (the masked) B
        let prepped_b_producer = XorGateVec::new(params.word_bits, creator);
        creator.multilink(&masked_b.output[], &prepped_b_producer.a[], STANDARD_DELAY);
        creator.link_one_to_many(do_invert, &prepped_b_producer.b[], STANDARD_DELAY);
        
        // Add the tweaked B with A
        let adder = RippleCarryAdder::new(creator, params.word_bits);
        creator.multilink(&prepped_b_producer.output[], &adder.b[], STANDARD_DELAY);
        let a_raw_input = &adder.a;
        creator.link(adder.carry_in, carry_in_set, STANDARD_DELAY);
        
        
        //
        // Build the ander-using branch
        //
        let ander = AndGateVec::new(params.word_bits, creator);
        creator.multilink(&a_raw_input[], &ander.a[], STANDARD_DELAY);
        creator.multilink(&masked_b.output[], &ander.b[], STANDARD_DELAY);
        
        
        //
        // Build the adder-vs-ander chooser
        //
        let chooser = Mux::new(params.word_bits, creator);
        creator.multilink(&adder.sum[], &chooser.a[], STANDARD_DELAY);
        creator.multilink(&ander.output[], &chooser.b[], STANDARD_DELAY);
        creator.link(select_and, chooser.select, STANDARD_DELAY);
        
        
        Alu {
            a: a_raw_input.clone(),
            b: b_raw_input.clone(),
            mode: control_rom.address,
            output: chooser.output,
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::super::truth_table::check_truth_table;
    use super::super::super::sim::{NodeCreator};
    use super::{Alu};
    use super::super::params::Params;
    
    #[test]
    fn alu_ops() {
        check_truth_table(|creator: &mut NodeCreator| {
            let params = Params{
                word_bits: 4,
                log_register_count: 8,
            };
            let alu = Alu::new(&params, creator);
            
            let mut inputs = Vec::new();
            inputs.append(&mut alu.mode.clone());
            inputs.append(&mut alu.a.clone());
            inputs.append(&mut alu.b.clone());
            
            (inputs, alu.output.clone())
        }, &[
            (&[0,0,0, 1,1,1,1, 0,0,1,0], &[0,0,0,0]), // zero
            (&[1,0,0, 1,1,1,0, 0,0,1,0], &[1,1,1,0]), // identity
            (&[0,1,0, 1,0,1,1, 0,0,1,0], &[0,1,1,1]), // increment
            (&[1,1,0, 1,0,1,1, 0,0,1,0], &[0,0,1,1]), // decrement
            (&[0,0,1, 1,0,1,0, 0,0,1,0], &[1,0,0,1]), // add
            (&[1,0,1, 1,0,1,0, 0,0,1,0], &[1,0,0,0]), // subtract
            (&[0,1,1, 1,0,1,0, 0,0,1,0], &[0,0,1,0]), // and
        ]);
    }
   
}
