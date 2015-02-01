#![feature(core, rustc_private)]

extern crate arena;

use arena::Arena;

use std::collections::binary_heap::BinaryHeap;
use std::cmp::PartialOrd;
use std::cmp::{Ord, Ordering};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
}

#[derive(Debug, Copy, Clone)]
struct Link {
    linked_to: NodeIndex,
    delay: PropogationDelay,
    id: u64,
}

#[derive(Debug, Copy, Clone)]
struct Influence {
    force_generator: NodeIndex,
    force_kind: LineState,
    force_id: u64,
}

struct Node {
    output_state: LineState,
    linked_with: Vec<Link>,
    element_index: Option<ElementIndex>,
    influences: Vec<Influence>
}

impl Node {
    fn new() -> Node {
        Node {
            output_state: LineState::Floating,
            linked_with: Vec::new(),
            element_index: None,
            influences: Vec::new(),
        }
    }
    
    fn get_input_state(&self) -> LineState {
        let mut lows = 0;
        let mut highs = 0;
        
        for influence in self.influences.iter() {
            let (delta_low, delta_high) = influence.force_kind.lows_highs_count();
            lows += delta_low;
            highs += delta_high;
        }
    
        match (lows>0, highs>0) {
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
    new_state: LineState,
    time: u64,
    id: u64, // to keep time ties in order
    forcer: NodeIndex,
    force_id: u64,
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
    link_id_counter: u64,
    force_id_counter: u64,
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
            link_id_counter: 0,
            force_id_counter: 0,
        }
    }
    
    fn link(&mut self, a: NodeIndex, b: NodeIndex, delay: PropogationDelay) {
        self.link_id_counter += 1;
        self.nodes[a.get()].linked_with.push(Link{linked_to: b, delay: delay, id: self.link_id_counter});
        self.nodes[b.get()].linked_with.push(Link{linked_to: a, delay: delay, id: self.link_id_counter});
    }
    
    fn apply_influence(&mut self, e: &LineStateEvent) {
    
        let target = &mut self.nodes[e.node.get()];
        if let Some(ref mut existing_influence) = target.influences.iter_mut().find(|&: influence| { influence.force_generator == e.forcer }) {
            if existing_influence.force_id >= e.force_id {
                return;
            }
            existing_influence.force_id = e.force_id;
            existing_influence.force_kind = e.new_state;
            return;
        }
        
        // No influence on this node so far.
        target.influences.push(Influence{
            force_generator: e.forcer,
            force_id: e.force_id,
            force_kind: e.new_state,
        });
    }
    
    fn play_event(&mut self, e: LineStateEvent) {
        self.current_tick = e.time;
        self.apply_influence(&e);
        
        let target = &self.nodes[e.node.get()];
        
        for adjacent_link in target.linked_with.iter() {
            let adjacent_node = &self.nodes[adjacent_link.linked_to.get()];
            let already_influenced = 
                if let Some(existing) = adjacent_node.influences.iter().find(|&: influence| { influence.force_generator == e.forcer }) {
                    existing.force_id >= e.force_id
                } else {
                    false
                };
            if !already_influenced {
                self.event_id_counter += 1;
                
                println!("Propogating from {:?} to {:?} at time {:?} with delay {:?}", e.node, adjacent_link.linked_to, self.current_tick, adjacent_link.delay.get());
                let evt = LineStateEvent{
                    node: adjacent_link.linked_to,
                    new_state: e.new_state,
                    time: self.current_tick + adjacent_link.delay.get() as u64,
                    id: self.event_id_counter,
                    forcer: e.forcer,
                    force_id: e.force_id,
                };
                self.events.push(evt);
            }
        }
    }
    
    fn play(&mut self) -> bool {
        if let Some(evt) = self.events.pop() {
            println!("Playing event: {:?}", evt);
            
            let maybe_element_index = self.nodes[evt.node.get()].element_index;
            self.play_event(evt);
            
            if let Some(element_index) = maybe_element_index{
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
        c.event_id_counter += 1;
        c.force_id_counter += 1;
        
        let evt = LineStateEvent{
            node: self,
            new_state: new_state,
            time: c.current_tick,
            id: c.event_id_counter,
            forcer: self,
            force_id: c.force_id_counter,
        };
        c.events.push(evt);
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

#[derive(Debug)]
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
    
    
    fn new_logtime<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena, input_count: usize) -> NWayAnd {
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

fn main() {
    let mut arena: Arena = Arena::new();
    
    let mut c = NodeCollection::new();
    let mut creator = NodeCreator::new(&c);
    let power = arena.alloc(|| { Pin::new(&mut creator) });
    let ground = arena.alloc(|| { Pin::new(&mut creator) });
    let overall_output = arena.alloc(|| { Pin::new(&mut creator) });
    
    let and1 = AndGate::new(&mut creator, &arena);
    let and2 = AndGate::new(&mut creator, &arena);
    let big_nander = NWayAnd::new_logtime(&mut creator, &arena, 50);
    
    println!("{:?}", big_nander.inputs);
    for (i, big_nander_input) in big_nander.inputs.iter().enumerate() {
        let high = true;
        creator.link(if high {power.node} else {ground.node}, *big_nander_input, STANDARD_DELAY);
    }
    
    creator.link(big_nander.output, overall_output.node, STANDARD_DELAY);
    /*
    //creator.link(power.node, and2.a, STANDARD_DELAY);
    
    creator.link(and2.a, and1.a, STANDARD_DELAY);
    println!("And1 = {:?}", and1);
    println!("And2 = {:?}", and2);
    creator.link(power.node, and2.a, STANDARD_DELAY);
    creator.link(power.node, and1.a, STANDARD_DELAY);
    creator.link(ground.node, and1.b, STANDARD_DELAY);
    creator.link(and1.output, overall_output.node, STANDARD_DELAY);
    */
    
    c.absorb(creator);
    
    c.add_element(power);
    c.add_element(ground);
    c.add_element(overall_output);
    
    power.node.write(LineState::High, &mut c);
    ground.node.write(LineState::Low, &mut c);
    
    loop {
        let more = c.play();
        if !more { break; }
    }
    
    println!("settled at t={}. out = {:?}", c.current_tick, overall_output.node.read(&c));
}