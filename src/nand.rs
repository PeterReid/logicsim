use sim::{LineState, NodeIndex, NodeCreator, Element, NodeCollection};

#[derive(Debug)]
pub struct NandElem {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
}

impl NandElem {
    pub fn new(c: &mut NodeCreator) -> NandElem {
        NandElem {
            a: c.new_node(),
            b: c.new_node(),
            output: c.new_node(),
        }
    }
}

impl Element for NandElem {
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
