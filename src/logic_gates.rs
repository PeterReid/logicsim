use sim::{NodeIndex, NodeCreator, PropogationDelay, STANDARD_DELAY};
use nand::NandElem;

use arena::Arena;

#[derive(Debug)]
pub struct NandGate {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
}

impl NandGate {
    pub fn new(creator: &mut NodeCreator) -> NandGate {
        let elem = creator.arena.alloc(|| { NandElem::new(creator) });
        creator.add_element(elem);
        
        NandGate {
            a: elem.a,
            b: elem.b,
            output: elem.output
        }
    }
}

#[derive(Debug)]
pub struct AndGate {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
}

impl AndGate {
    pub fn new(creator: &mut NodeCreator) -> AndGate {
        let nander = NandGate::new(creator);
        let notter = NandGate::new(creator);
        
        creator.link(nander.output, notter.a, STANDARD_DELAY);
        creator.link(nander.output, notter.b, STANDARD_DELAY);
        
        AndGate {
            a: nander.a,
            b: nander.b,
            output: notter.output,
        }
    }
}

pub struct AndGateVec {
    pub a: Vec<NodeIndex>,
    pub b: Vec<NodeIndex>,
    pub output: Vec<NodeIndex>,
}
impl AndGateVec {
    pub fn new(count: usize, creator: &mut NodeCreator) -> AndGateVec {
        let subgates : Vec<AndGate> = range(0, count).map(|_| { AndGate::new(creator) }).collect();
        AndGateVec {
            a: subgates.iter().map(|gate| { gate.a }).collect(),
            b: subgates.iter().map(|gate| { gate.b }).collect(),
            output: subgates.iter().map(|gate| { gate.output }).collect(),
        }
    }
}

pub struct NotGate {
    pub input: NodeIndex,
    pub output: NodeIndex,
}

impl NotGate {
    pub fn new(creator: &mut NodeCreator) -> NotGate {
        let nand = NandGate::new(creator);
        creator.link(nand.a, nand.b, STANDARD_DELAY);
        NotGate {
            input: nand.a,
            output: nand.output
        }
    }
}

#[allow(dead_code)]
pub struct NWayAnd {
    inputs: Vec<NodeIndex>,
    output: NodeIndex,
}

#[allow(dead_code)]
impl NWayAnd {
    pub fn new(creator: &mut NodeCreator, input_count: usize) -> NWayAnd {
        if input_count < 2 {
            panic!("NWayAnd needs at least 2 inputs");
        }
        
        let mut inputs = Vec::new();
        
        let and0 = AndGate::new(creator);
        inputs.push(and0.a);
        inputs.push(and0.b);
        let mut output_so_far = and0.output;
        
        for _ in range(2, input_count) {
            let and = AndGate::new(creator);
            creator.link(output_so_far, and.a, STANDARD_DELAY);
            output_so_far = and.output;
            inputs.push(and.b);
        }
        
        NWayAnd {
            inputs: inputs,
            output: output_so_far
        }
    }
    
    
    pub fn new_logtime(creator: &mut NodeCreator, input_count: usize) -> NWayAnd {
        if input_count == 0 {
            panic!("Can't have an NWayAnd with no inputs!");
        }
         
        let inputs : Vec<NodeIndex> = range(0, input_count).map(|_| { creator.new_node() }).collect();
        let mut frontier : Vec<(NodeIndex, PropogationDelay)> = inputs.iter().map(|input| { (*input, PropogationDelay(0)) }).collect();
        while frontier.len() > 1 {
            println!("{:?}", frontier);
            let mut next_frontier = Vec::new();
            
            for pair in frontier.as_slice().chunks(2) {
                if pair.len() == 2 {
                    let and = AndGate::new(creator);
                    let (node_a, delay_a) = pair[0];
                    let (node_b, delay_b) = pair[1];
                    creator.link(node_a, and.a, delay_a);
                    creator.link(node_b, and.b, delay_b);
                    next_frontier.push((and.output, STANDARD_DELAY));
                } else {
                    next_frontier.push(pair[0]);
                }
            }
            
            frontier = next_frontier;
        }
        
        let (last_node, _) = frontier[0];
        
        NWayAnd {
            inputs: inputs,
            output: last_node
        }
    }
}


pub struct XorGate {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
}

impl XorGate {
    pub fn new(creator: &mut NodeCreator) -> XorGate {
        let a_nand_b = NandGate::new(creator);
        let top = NandGate::new(creator);
        let bottom = NandGate::new(creator);
        let output = NandGate::new(creator);
        
        creator.link(a_nand_b.output, top.b, STANDARD_DELAY);
        creator.link(a_nand_b.output, bottom.a, STANDARD_DELAY);
        creator.link(a_nand_b.a, top.a, STANDARD_DELAY);
        creator.link(a_nand_b.b, bottom.b, STANDARD_DELAY);
        creator.link(top.output, output.a, STANDARD_DELAY);
        creator.link(bottom.output, output.b, STANDARD_DELAY);
        
        XorGate {
            a: a_nand_b.a,
            b: a_nand_b.b,
            output: output.output,
        }
    }
}

pub struct XorGateVec {
    pub a: Vec<NodeIndex>,
    pub b: Vec<NodeIndex>,
    pub output: Vec<NodeIndex>,
}
impl XorGateVec {
    pub fn new(count: usize, creator: &mut NodeCreator) -> XorGateVec {
        let subgates : Vec<XorGate> = range(0, count).map(|_| { XorGate::new(creator) }).collect();
        XorGateVec {
            a: subgates.iter().map(|gate| { gate.a }).collect(),
            b: subgates.iter().map(|gate| { gate.b }).collect(),
            output: subgates.iter().map(|gate| { gate.output }).collect(),
        }
    }
}

pub struct OrGate {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
}

impl OrGate {
    pub fn new(creator: &mut NodeCreator) -> OrGate {
        let not_a = NandGate::new(creator);
        let not_b = NandGate::new(creator);
        let or = NandGate::new(creator);
        
        creator.link(not_a.a, not_a.b, STANDARD_DELAY);
        creator.link(not_b.a, not_b.b, STANDARD_DELAY);
        creator.link(not_a.output, or.a, STANDARD_DELAY);
        creator.link(not_b.output, or.b, STANDARD_DELAY);
        
        OrGate {
            a: not_a.a,
            b: not_b.b,
            output: or.output,
        }
    }
}
