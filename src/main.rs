#![feature(core)]

use std::collections::binary_heap::BinaryHeap;
use std::cmp::PartialOrd;
use std::cmp::{Ord, Ordering};

#[derive(Debug, PartialEq, Eq, Copy)]
enum LineState {
    Low,
    High,
    Floating,
    Conflict,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeIndex(pub usize);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PropogationDelay(pub u32);

impl PropogationDelay {
    fn get(self) -> u32 {
        let PropogationDelay(ticks) = self;
        ticks
    }
}

struct Node {
    lows: u32,
    highs: u32,
    output_state: LineState,
    linked_with: Vec<(NodeIndex, PropogationDelay)>,
}

impl Node {
    fn new() -> Node {
        Node {
            lows: 0,
            highs: 0,
            output_state: LineState::Floating,
            linked_with: Vec::new(),
        }
    }
    
    fn get_input_state(&self) -> LineState {
        match (self.lows>0, self.highs>0) {
            (false, false) => LineState::Floating,
            (true, false) => LineState::Low,
            (false, true) => LineState::High,
            (true, true) => LineState::Conflict,
        }
    }
}

#[derive(PartialEq, Eq, Copy)]
struct LineStateEvent {
    node: NodeIndex,
    old_state: LineState,
    new_state: LineState,
    time: u64,
}

impl PartialOrd<LineStateEvent> for LineStateEvent {
    fn partial_cmp(&self, other: &LineStateEvent) -> Option<Ordering> {
        other.time.partial_cmp(&self.time)
    }
}
impl Ord for LineStateEvent {
    fn cmp(&self, other: &LineStateEvent) -> Ordering {
        other.time.cmp(&self.time)
    }
}

struct NodeCollection {
    nodes: Vec<Node>,
    events: BinaryHeap<LineStateEvent>,
    current_tick: u64
}

impl NodeCollection {
    fn new() -> NodeCollection {
        NodeCollection {
            nodes: Vec::new(),
            events: BinaryHeap::new(),
            current_tick: 0,
        }
    }
    fn new_node(&mut self) -> NodeIndex {
        self.nodes.push(Node::new());
        return NodeIndex(self.nodes.len() - 1);
    }
    
    fn link(&mut self, a: NodeIndex, b: NodeIndex, delay: PropogationDelay) {
        self.nodes[a.get()].linked_with.push((b, delay));
        self.nodes[b.get()].linked_with.push((b, delay));
    }
}

impl NodeIndex {
    fn get(self) -> usize {
        let NodeIndex(idx) = self;
        idx
    }

    fn write(self, value: LineState, c: &mut NodeCollection) {
        let node = &mut c.nodes[self.get()];
        let old_state = node.output_state;
        node.output_state = value;
        let now = c.current_tick;
        c.events.extend(node.linked_with.iter().map(|&: link: &(NodeIndex, PropogationDelay)| {
            let (linked_to, delay) = *link;
            LineStateEvent{
                node: linked_to,
                old_state: old_state,
                new_state: value,
                time: now + delay.get() as u64,
            }
        }));
    }
    
    fn read(self, c: &NodeCollection) -> LineState {
        c.nodes[self.get()].get_input_state()
    }
}

struct Transistor {
    pub input: NodeIndex,
    pub output: NodeIndex,
    pub enable: NodeIndex,
}

impl Transistor {
    fn new(c: &mut NodeCollection) -> Transistor {
        Transistor {
            input: c.new_node(),
            output: c.new_node(),
            enable: c.new_node(),
        }
    }
    
    fn step(&self, c: &mut NodeCollection) {
        self.output.write( match (self.enable.read(c), self.input.read(c)) {
            (LineState::Low, _) => LineState::Floating,
            (LineState::High, input_state) => input_state,
            (LineState::Floating, LineState::Floating) => LineState::Floating,
            (LineState::Floating, _) => LineState::Conflict,
            (LineState::Conflict, _) => LineState::Conflict,
        }, c);
    }
}

fn main() {
    let mut c = NodeCollection::new();
    
    let t1 = Transistor::new(&mut c);
    let t2 = Transistor::new(&mut c);
    
    c.link(t1.output, t2.enable, PropogationDelay(100));
    
    
    t1.step(&mut c);
    
    //let arena = Arena::new();
    //let mut t1 = arena.alloc(|| { Transistor::new() });
    //let mut t2 = arena.alloc(|| { Transistor::new() });
    //let &mut t2 = Transistor::new();
    
    //t1.output.link_to(&mut t2.enable, 100);
    
    
}