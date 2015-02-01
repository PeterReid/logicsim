use sim::{NodeIndex, NodeCreator, NodeCollection, Element};

pub struct Pin {
    pub node: NodeIndex
}

impl Pin {
    pub fn new(c: &mut NodeCreator) -> Pin {
        Pin {
            node: c.new_node(),
        }
    }
}

impl Element for Pin {
    fn step(&self, _: &mut NodeCollection) {
    }
    
    fn get_nodes(&self) -> Vec<NodeIndex> {
        let mut v = Vec::new();
        v.push(self.node);
        v
    }
}
