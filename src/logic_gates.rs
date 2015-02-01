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
    pub fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> NandGate {
        let elem = arena.alloc(|| { NandElem::new(creator) });
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
    pub fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> AndGate {
        let nander = NandGate::new(creator, arena);
        let notter = NandGate::new(creator, arena);
        
        creator.link(nander.output, notter.a, STANDARD_DELAY);
        creator.link(nander.output, notter.b, STANDARD_DELAY);
        
        AndGate {
            a: nander.a,
            b: nander.b,
            output: notter.output,
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
    pub fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena, input_count: usize) -> NWayAnd {
        if input_count < 2 {
            panic!("NWayAnd needs at least 2 inputs");
        }
        
        let mut inputs = Vec::new();
        
        let and0 = AndGate::new(creator, arena);
        inputs.push(and0.a);
        inputs.push(and0.b);
        let mut output_so_far = and0.output;
        
        for _ in range(2, input_count) {
            let and = AndGate::new(creator, arena);
            creator.link(output_so_far, and.a, STANDARD_DELAY);
            output_so_far = and.output;
            inputs.push(and.b);
        }
        
        NWayAnd {
            inputs: inputs,
            output: output_so_far
        }
    }
    
    
    pub fn new_logtime<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena, input_count: usize) -> NWayAnd {
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
                    let and = AndGate::new(creator, arena);
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
