use sim::{NodeIndex, STANDARD_DELAY, NodeCreator};

use arena::Arena;

use logic_gates::{XorGate, AndGate, OrGate};

pub struct HalfAdder {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub sum: NodeIndex,
    pub carry: NodeIndex,
}

impl HalfAdder {
    pub fn new(creator: &mut NodeCreator) -> HalfAdder {
        let different = XorGate::new(creator);
        let both = AndGate::new(creator);
        
        creator.link(different.a, both.a, STANDARD_DELAY);
        creator.link(different.b, both.b, STANDARD_DELAY);
        HalfAdder {
            a: different.a,
            b: different.b,
            sum: different.output,
            carry: both.output,
        }
    }
}

pub struct Adder {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub carry_in: NodeIndex,
    pub sum: NodeIndex,
    pub carry_out: NodeIndex,
}

impl Adder {
    pub fn new(creator: &mut NodeCreator) -> Adder {
        let half_one = HalfAdder::new(creator);
        let half_two = HalfAdder::new(creator);
        let either_carry = OrGate::new(creator);
        
        creator.link(half_one.carry, either_carry.a, STANDARD_DELAY);
        creator.link(half_two.carry, either_carry.b, STANDARD_DELAY);
        creator.link(half_one.sum, half_two.a, STANDARD_DELAY);
        
        Adder {
            a: half_one.a,
            b: half_one.b,
            carry_in: half_two.b,
            sum: half_two.sum,
            carry_out: either_carry.output,
        }
    }
}

pub struct RippleCarryAdder {
    a: Vec<NodeIndex>,
    b: Vec<NodeIndex>,
    carry_in: NodeIndex,
    
    sum: Vec<NodeIndex>,
    carry_out: NodeIndex,
}
impl RippleCarryAdder {
    pub fn new(creator: &mut NodeCreator, bits: usize) -> RippleCarryAdder {
        let adders : Vec<Adder> = range(0, bits).map(|_| { Adder::new(creator) }).collect();
        
        for idx in range(1, bits) {
            creator.link(adders[idx-1].carry_out, adders[idx].carry_in, STANDARD_DELAY);
        }
        
        RippleCarryAdder {
            a: adders.iter().map(|&: adder| { adder.a }).collect(),
            b: adders.iter().map(|&: adder| { adder.b }).collect(),
            sum: adders.iter().map(|&: adder| { adder.sum }).collect(),
            carry_in: adders[0].carry_in,
            carry_out: adders[adders.len()-1].carry_out,
        }
    }
}

#[cfg(test)]
mod test {
    use truth_table::check_truth_table;
    use sim::{NodeCreator};
    use super::{HalfAdder, Adder, RippleCarryAdder};
    
    #[test]
    fn test_half_adder() {
        
        check_truth_table(|creator: &mut NodeCreator| {
            let h = HalfAdder::new(creator);
            
            ([h.a, h.b].to_vec(), [h.sum, h.carry].to_vec())
        }, &[
            (&[0,0], &[0,0]),
            (&[1,0], &[1,0]),
            (&[0,1], &[1,0]),
            (&[1,1], &[0,1]),
        ]);
    }


    #[test]
    fn test_add() {
        check_truth_table(|creator: &mut NodeCreator| {
            let h = Adder::new(creator);
            
            ([h.a, h.b, h.carry_in].to_vec(), [h.sum, h.carry_out].to_vec())
        }, &[
            (&[0,0,0], &[0,0]),
            (&[1,0,0], &[1,0]),
            (&[0,1,0], &[1,0]),
            (&[1,1,0], &[0,1]),
            (&[0,0,1], &[1,0]),
            (&[1,0,1], &[0,1]),
            (&[0,1,1], &[0,1]),
            (&[1,1,1], &[1,1]),
        ]);
    }

    #[test]
    fn test_4_bit_add() {
        check_truth_table(|creator: &mut NodeCreator| {
            let h = RippleCarryAdder::new(creator, 4);
            
            let mut inputs = Vec::new();
            inputs.push(h.carry_in);
            inputs.append(&mut h.a.clone());
            inputs.append(&mut h.b.clone());
            
            let mut outputs = h.sum.clone();
            outputs.push(h.carry_out);
            
            (inputs, outputs)
        }, &[
            (&[0, 0,0,0,0, 0,0,0,0], &[0,0,0,0,0]),
            (&[0, 1,0,0,0, 0,0,0,0], &[1,0,0,0,0]),
            (&[0, 1,1,1,0, 1,0,0,0], &[0,0,0,1,0]),
            (&[0, 1,1,1,1, 1,1,1,1], &[0,1,1,1,1]),
            (&[1, 1,1,1,1, 1,1,1,1], &[1,1,1,1,1]),
        ]);
    }
}