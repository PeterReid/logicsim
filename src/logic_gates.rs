use sim::{LineState, NodeIndex, NodeCreator, Element, NodeCollection, PropogationDelay, STANDARD_DELAY};

use arena::Arena;

#[derive(Debug)]
pub struct Nand {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
}

impl Nand {
    pub fn new(c: &mut NodeCreator) -> Nand {
        Nand {
            a: c.new_node(),
            b: c.new_node(),
            output: c.new_node(),
        }
    }
}

impl Element for Nand {
    fn step(&self, c: &mut NodeCollection) {
        
        let res = match (self.a.read(c), self.b.read(c)) {
            (LineState::Floating, _) => LineState::Floating, // not sure if this is physically accurate
            (_, LineState::Floating) => LineState::Floating, // not sure if this is physically accurate
            (LineState::Conflict, _) => LineState::Conflict,
            (_, LineState::Conflict) => LineState::Conflict,
            (LineState::High, LineState::High) => LineState::Low,
            _ => LineState::High
        };
        //println!("Running nand {:?}: {:?} {:?} -> {:?}", self, self.a.read(c), self.b.read(c), res);
        self.output.write(res , c);
    }
    
    fn get_nodes(&self) -> Vec<NodeIndex> {
        let mut v = Vec::new();
        v.push(self.a);
        v.push(self.b);
        v.push(self.output);
        v
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
        let nander = arena.alloc(|| { Nand::new(creator) });
        let notter = arena.alloc(|| { Nand::new(creator) });
        
        creator.add_element(nander);
        creator.add_element(notter);
        
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
