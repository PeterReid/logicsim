#![feature(core, rustc_private)]

extern crate arena;

use arena::Arena;

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

impl LineState {
    fn lows_highs_count(self) -> (u32, u32) {
        match self {
            LineState::Low => (1, 0),
            LineState::High => (0, 1),
            LineState::Floating => (0, 0),
            LineState::Conflict => (1, 1),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeIndex(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ElementIndex(pub usize);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PropogationDelay(pub u32);

impl PropogationDelay {
    fn get(self) -> u32 {
        let PropogationDelay(ticks) = self;
        ticks
    }
}

trait Element {
    fn step(&self, c: &mut NodeCollection);
    fn get_nodes(&self) -> Vec<NodeIndex>;
    //fn with_nodes<F>(&mut self, f: F)
        //where F: FnOnce(&[&mut NodeIndex]);
}

struct Node {
    lows: u32,
    highs: u32,
    output_state: LineState,
    linked_with: Vec<(NodeIndex, PropogationDelay)>,
    element_index: Option<ElementIndex>,
}

impl Node {
    fn new() -> Node {
        Node {
            lows: 0,
            highs: 0,
            output_state: LineState::Floating,
            linked_with: Vec::new(),
            element_index: None,
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

#[derive(PartialEq, Eq, Copy, Debug)]
struct LineStateEvent {
    node: NodeIndex,
    old_state: LineState,
    new_state: LineState,
    time: u64,
    id: u64, // to keep time ties in order
}

impl PartialOrd<LineStateEvent> for LineStateEvent {
    fn partial_cmp(&self, other: &LineStateEvent) -> Option<Ordering> {
        if other.time != self.time {
            other.time.partial_cmp(&self.time)
        } else {
            other.id.partial_cmp(&self.id)
        }
    }
}
impl Ord for LineStateEvent {
    fn cmp(&self, other: &LineStateEvent) -> Ordering {
        if other.time != self.time {
            other.time.cmp(&self.time)
        } else {
            other.id.cmp(&self.id)
        }
    }
}

struct NodeCollection<'a> {
    nodes: Vec<Node>,
    events: BinaryHeap<LineStateEvent>,
    current_tick: u64,
    elements: Vec<&'a (Element + 'a)>,
    //element_arena: &'a Arena,
    event_id_counter: u64,
}

impl<'a> NodeCollection<'a> {
    fn new() -> NodeCollection<'a> {
        NodeCollection {
            //element_arena: arena,
            nodes: Vec::new(),
            events: BinaryHeap::new(),
            current_tick: 0,
            elements: Vec::new(),
            event_id_counter: 0,
        }
    }
    
    fn link(&mut self, a: NodeIndex, b: NodeIndex, delay: PropogationDelay) {
        self.nodes[a.get()].linked_with.push((b, delay));
        self.nodes[b.get()].linked_with.push((a, delay));
    }
    
    fn play(&mut self) -> bool {
        if let Some(evt) = self.events.pop() {
            println!("Playing event: {:?}", evt);
            self.current_tick = evt.time;
            if let Some(element_index) = {
                let affected_node = &mut self.nodes[evt.node.get()];
                let (old_low, old_high) = evt.old_state.lows_highs_count();
                let (new_low, new_high) = evt.new_state.lows_highs_count();
                affected_node.lows += new_low - old_low;
                affected_node.highs += new_high - old_high;
                affected_node.element_index
            } {
                self.elements[element_index.get()].step(self);
            }
            return true;
        } else {
            println!("No events");
            return false;
        }
    }
    
    fn absorb<'b:'a>(&mut self, creator: NodeCreator<'b>) {
        for element in creator.elements.iter() {
            self.add_element(*element);
        }
        
        for &(a, b, delay) in creator.links.iter() {
            self.link(a, b, delay);
        }
    }
    
    fn add_element(&mut self, elem: &'a (Element + 'a)) {
    
        let element_index = ElementIndex(self.elements.len());
        for node_index in elem.get_nodes().iter() {
            
            while self.nodes.len() <= node_index.get() {
                self.nodes.push(Node::new());
            }
        
            let node = &mut self.nodes[node_index.get()];
            assert!(node.element_index.is_none(), "An element tried to claim an already-claimed node!");
            node.element_index = Some(element_index);
        }
    
        self.elements.push(elem);
        
    }
}

impl ElementIndex {
    fn get(self) -> usize {
        let ElementIndex(idx) = self;
        idx
    }
}

impl NodeIndex {
    fn get(self) -> usize {
        let NodeIndex(idx) = self;
        idx
    }

    fn set(&mut self, val: usize) {
        match self {
            &mut NodeIndex(ref mut x) => {
                *x = val;
            }
        }
    }
    
    fn write(self, new_state: LineState, c: &mut NodeCollection) {
        let node = &mut c.nodes[self.get()];
        
        let old_state = node.output_state;
        if new_state == old_state {
            return; // no-op
        }
        
        node.output_state = new_state;
        let now = c.current_tick;
        let mut id_counter = c.event_id_counter;
        
        id_counter += 1;
        c.events.push(LineStateEvent{
            node: self,
            old_state: old_state,
            new_state: new_state,
            time: now,
            id: id_counter
        });
        
        c.events.extend(node.linked_with.iter().map(|&mut : link: &(NodeIndex, PropogationDelay)| {
            let (linked_to, delay) = *link;
            id_counter += 1;
            LineStateEvent{
                node: linked_to,
                old_state: old_state,
                new_state: new_state,
                time: now + delay.get() as u64,
                id: id_counter,
            }
        }));
        c.event_id_counter = id_counter;
    }
    
    fn read(self, c: &NodeCollection) -> LineState {
        c.nodes[self.get()].get_input_state()
    }
}

#[derive(Debug)]
struct Nand {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
}

struct NodeCreator<'a> {
    creation_index: usize,
    elements: Vec<&'a (Element + 'a)>,
    links: Vec<(NodeIndex, NodeIndex, PropogationDelay)>,
}

impl<'a> NodeCreator<'a> {

    fn new(parent: &NodeCollection<'a>) -> NodeCreator<'a> {
        NodeCreator{
            creation_index: parent.nodes.len(),
            elements: Vec::new(),
            links: Vec::new()
        }
    }
    
    fn new_node(&mut self) -> NodeIndex {
        let ret = self.creation_index;
        self.creation_index += 1;
        NodeIndex(ret)
    }
    
    fn add_element(&mut self, elem: &'a (Element + 'a)) {
        self.elements.push(elem);
    }
    
    fn link(&mut self, a: NodeIndex, b: NodeIndex, delay: PropogationDelay) {
        self.links.push((a, b, delay));
    }
}

impl Nand {
    fn new(c: &mut NodeCreator) -> Nand {
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
        println!("Running nand {:?}: {:?} {:?} -> {:?}", self, self.a.read(c), self.b.read(c), res);
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

struct Pin {
    node: NodeIndex
}

impl Pin {
    fn new(c: &mut NodeCreator) -> Pin {
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

const STANDARD_DELAY: PropogationDelay = PropogationDelay(100);

struct AndGate {
    a: NodeIndex,
    b: NodeIndex,
    output: NodeIndex,
}

impl AndGate {
    fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> AndGate {
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

struct NWayAnd {
    inputs: Vec<NodeIndex>,
    output: NodeIndex,
}

impl NWayAnd {
    fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena, input_count: usize) -> NWayAnd {
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
}

fn main() {
    let mut arena: Arena = Arena::new();
    
    let mut c = NodeCollection::new();
    let mut creator = NodeCreator::new(&c);
    let power = arena.alloc(|| { Pin::new(&mut creator) });
    let ground = arena.alloc(|| { Pin::new(&mut creator) });
    let overall_output = arena.alloc(|| { Pin::new(&mut creator) });
    //let t1 = arena.alloc(|| { Nand::new(&mut creator) });
    //let t2 = arena.alloc(|| { Nand::new(&mut creator) });
    
    let and1 = AndGate::new(&mut creator, &arena);
    let and2 = AndGate::new(&mut creator, &arena);
    let big_nander = NWayAnd::new(&mut creator, &arena, 20);
    
    println!("{:?}", big_nander.inputs);
    for (i, big_nander_input) in big_nander.inputs.iter().enumerate() {
        let high = true;
        creator.link(if high {power.node} else {ground.node}, *big_nander_input, STANDARD_DELAY);
    }
    
    creator.link(big_nander.output, overall_output.node, STANDARD_DELAY);
    
    //creator.link(ground.node, and1.a, STANDARD_DELAY);
    //creator.link(ground.node, and1.b, STANDARD_DELAY);
    //creator.link(and1.output, overall_output.node, STANDARD_DELAY);
    c.absorb(creator);
    
    //let power = c.new_node();
    //let ground = c.new_node();
    
    c.add_element(power);
    c.add_element(ground);
    //c.add_element(t1);
    //c.add_element(t2);
    
    //c.link(power.node, t1.a, PropogationDelay(100));
    //c.link(power.node, t1.b, PropogationDelay(100));
    //c.link(t1.output, t2.a, PropogationDelay(120));
    //c.link(t2.b, power.node, PropogationDelay(100));
    //c.link(t2.output, overall_output.node, PropogationDelay(40));
    
    power.node.write(LineState::High, &mut c);
    ground.node.write(LineState::Low, &mut c);
    
    loop {
        let more = c.play();
        if !more { break; }
    }
    
    println!("settled at t={}. out = {:?}", c.current_tick, overall_output.node.read(&c));
    
    //let t1 = Transistor::new(&mut c);
    //let t2 = Transistor::new(&mut c);
    
    //c.link(t1.output, t2.enable, PropogationDelay(100));
    
    //c.elements.push(t1);
    
    //t1.step(&mut c);
    
    
    
    //let mut t2 = arena.alloc(|| { Transistor::new() });
    //let &mut t2 = Transistor::new();
    
    //t1.output.link_to(&mut t2.enable, 100);
    
    
}