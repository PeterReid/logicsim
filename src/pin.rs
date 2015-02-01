use sim::{NodeIndex, NodeCreator, NodeCollection, Element};

use arena::Arena;

pub struct PinElem {
    pub node: NodeIndex
}

impl PinElem {
    pub fn new(c: &mut NodeCreator) -> PinElem {
        PinElem {
            node: c.new_node(),
        }
    }
}

impl Element for PinElem {
    fn step(&self, _: &mut NodeCollection) {
    }
    
    fn get_nodes(&self) -> Vec<NodeIndex> {
        let mut v = Vec::new();
        v.push(self.node);
        v
    }
}

pub struct Pin {
    pub node: NodeIndex
}

impl Pin {
    pub fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> Pin {
        let elem = arena.alloc(|| { PinElem::new(creator) });
        creator.add_element(elem);
        Pin {
            node: elem.node
        }
    }
}
