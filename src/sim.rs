
use std::collections::binary_heap::BinaryHeap;
use std::cmp::PartialOrd;
use std::cmp::{Ord, Ordering};

use arena::Arena;
use std::mem::transmute;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LineState {
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


pub trait Element {
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
            (false, false) => LineState::Low, // is this physically accurate? I am not sure
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

pub struct NodeCollection<'a> {
    nodes: Vec<Node>,
    events: BinaryHeap<LineStateEvent>,
    pub current_tick: u64,
    elements: Vec<&'a (Element + 'a)>,
    element_arena: Arena,
    event_id_counter: u64,
    link_id_counter: u64,
    force_id_counter: u64,
}

impl<'a> NodeCollection<'a> {
    pub fn new() -> NodeCollection<'a> {
        NodeCollection {
            element_arena: Arena::new(),
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
    
    unsafe fn static_arena_ref(&self) -> &'static Arena {
        let x : &'static Arena = transmute(&self.element_arena);
        x
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
                
                //println!("Propogating from {:?} to {:?} at time {:?} with delay {:?}", e.node, adjacent_link.linked_to, self.current_tick, adjacent_link.delay.get());
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
    
    pub fn play(&mut self) -> bool {
        if let Some(evt) = self.events.pop() {
            //println!("Playing event: {:?}", evt);
            
            let maybe_element_index = self.nodes[evt.node.get()].element_index;
            self.play_event(evt);
            
            if let Some(element_index) = maybe_element_index{
                self.elements[element_index.get()].step(self);
            }
            return true;
        } else {
            return false;
        }
    }
    
    pub fn absorb<'b:'a>(&mut self, creator: NodeCreator<'b>) {
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

    pub fn write(self, new_state: LineState, c: &mut NodeCollection) {
        if new_state == c.nodes[self.get()].output_state {
            return; // no-op
        }
        self.write_later(new_state, PropogationDelay(0), c)
    }
    
    pub fn write_later(self, new_state: LineState, delta_time: PropogationDelay, c: &mut NodeCollection) {
        let node = &mut c.nodes[self.get()];
        
        node.output_state = new_state;
        c.event_id_counter += 1;
        c.force_id_counter += 1;
        
        let evt = LineStateEvent{
            node: self,
            new_state: new_state,
            time: c.current_tick + delta_time.get() as u64,
            id: c.event_id_counter,
            forcer: self,
            force_id: c.force_id_counter,
        };
        c.events.push(evt);
    }
    
    pub fn read(self, c: &NodeCollection) -> LineState {
        c.nodes[self.get()].get_input_state()
    }
}


pub struct NodeCreator<'a> {
    creation_index: usize,
    elements: Vec<&'a (Element + 'a)>,
    links: Vec<(NodeIndex, NodeIndex, PropogationDelay)>,
    pub arena: &'static Arena,
}

impl<'a> NodeCreator<'a> {

    pub fn new(parent: &NodeCollection<'a>) -> NodeCreator<'a> {
        NodeCreator{
            creation_index: parent.nodes.len(),
            elements: Vec::new(),
            links: Vec::new(),
            arena: unsafe{ parent.static_arena_ref() },
        }
    }
    
    pub fn new_node(&mut self) -> NodeIndex {
        let ret = self.creation_index;
        self.creation_index += 1;
        NodeIndex(ret)
    }
    
    pub fn add_element(&mut self, elem: &'a (Element + 'a)) {
        self.elements.push(elem);
    }
    
    pub fn link(&mut self, a: NodeIndex, b: NodeIndex, delay: PropogationDelay) {
        self.links.push((a, b, delay));
    }
}

pub const STANDARD_DELAY: PropogationDelay = PropogationDelay(100);
