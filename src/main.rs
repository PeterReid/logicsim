#![feature(core, rustc_private)]

extern crate arena;

mod base;

use arena::Arena;

use std::collections::binary_heap::BinaryHeap;
use std::cmp::PartialOrd;
use std::cmp::{Ord, Ordering};
use std::collections::HashSet;

use base::{LineState, NodeIndex, NodeCreator, Element, NodeCollection, PropogationDelay};

#[derive(Debug)]
struct Nand {
    pub a: NodeIndex,
    pub b: NodeIndex,
    pub output: NodeIndex,
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

#[derive(Debug)]
struct NotSRLatch {
    not_s: NodeIndex,
    not_r: NodeIndex,
    q: NodeIndex,
    not_q: NodeIndex,
}

impl NotSRLatch {
    fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> NotSRLatch {
        let top = arena.alloc(|| { Nand::new(creator) });
        let bottom = arena.alloc(|| { Nand::new(creator) });
        
        creator.add_element(top);
        creator.add_element(bottom);
        
        creator.link(top.output, bottom.a, STANDARD_DELAY);
        creator.link(bottom.output, top.b, STANDARD_DELAY);
        
        NotSRLatch{
            not_s: top.a,
            not_r: bottom.b,
            q: top.output,
            not_q: bottom.output,
        }
    }
}


#[derive(Debug)]
struct DFlipFlop {
    clock: NodeIndex,
    data: NodeIndex,
    q: NodeIndex,
    not_q: NodeIndex,
}

impl DFlipFlop {
    fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena) -> DFlipFlop {
        let top = NotSRLatch::new(creator, arena);
        let bottom = NotSRLatch::new(creator, arena);
        let output = NotSRLatch::new(creator, arena);
        let ander = AndGate::new(creator, arena);
        
        let clock = ander.a;
        
        creator.link(ander.b, top.not_q, STANDARD_DELAY);
        creator.link(ander.output, bottom.not_s, STANDARD_DELAY);
        let data = bottom.not_r;
        
        creator.link(bottom.q, output.not_r, STANDARD_DELAY);
        creator.link(clock, top.not_r, STANDARD_DELAY);
        creator.link(bottom.not_q, top.not_s, STANDARD_DELAY);
        creator.link(top.not_q, output.not_s, STANDARD_DELAY);
        
        DFlipFlop {
            clock: clock,
            data: data,
            q: output.q,
            not_q: output.not_q
        }
    }
}

struct Register {
    bits: Vec<DFlipFlop>,
    clock: NodeIndex,
}

impl Register {
    fn new<'a, 'b:'a>(creator: &mut NodeCreator<'a>, arena: &'b Arena, bit_count: usize) -> Register {
        let bits : Vec<DFlipFlop> = range(0, bit_count).map(|_| { DFlipFlop::new(creator, arena) }).collect();
        
        let clock = bits[0].clock;
        for bit in bits.slice_from(1).iter() {
            creator.link(clock, bit.clock, STANDARD_DELAY);
        }
        
        Register {
            bits: bits,
            clock: clock,
        }
    }
    
    fn read_u64(&self, c: &NodeCollection) -> Option<u64> {
        let mut accum = 0u64;
        for (index, bit) in self.bits.iter().enumerate() {
            match bit.q.read(c) {
                LineState::Low => {},
                LineState::High => {
                    accum |= 1<<index;
                },
                _ => { return None; }
            }
        }
        
        return Some(accum);
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
    let clock = arena.alloc(|| { Pin::new(&mut creator) });
    let data = arena.alloc(|| { Pin::new(&mut creator) });
    
    //let not_s = arena.alloc(|| { Pin::new(&mut creator) });
    //let not_r = arena.alloc(|| { Pin::new(&mut creator) });
    
    creator.add_element(power);
    creator.add_element(ground);
    creator.add_element(overall_output);
    creator.add_element(clock);
    creator.add_element(data);
    //c.add_element(not_s);
    //c.add_element(not_r);
    
    //let sr1 = NotSRLatch::new(&mut creator, &arena);
    //creator.link(not_s.node, sr1.not_s, STANDARD_DELAY);
    //creator.link(not_r.node, sr1.not_r, STANDARD_DELAY);
    
    
    let d = DFlipFlop::new(&mut creator, &arena);
    creator.link(clock.node, d.clock, STANDARD_DELAY);
    creator.link(data.node, d.data, STANDARD_DELAY);
    
    
    let r = Register::new(&mut creator, &arena, 8);
    creator.link(clock.node, r.clock, STANDARD_DELAY);
    creator.link(power.node, r.bits[0].data, STANDARD_DELAY);
    
    /*let big_nander = NWayAnd::new_logtime(&mut creator, &arena, 50);
    
    println!("{:?}", big_nander.inputs);
    for (i, big_nander_input) in big_nander.inputs.iter().enumerate() {
        let high = true;
        creator.link(if high {power.node} else {ground.node}, *big_nander_input, STANDARD_DELAY);
    }
    
    creator.link(big_nander.output, overall_output.node, STANDARD_DELAY);
    */
    
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
    
    power.node.write(LineState::High, &mut c);
    ground.node.write(LineState::Low, &mut c);
    data.node.write(LineState::Low, &mut c);
    clock.node.write(LineState::Low, &mut c);
    
    clock.node.write_later(LineState::High, PropogationDelay(1000), &mut c);
    clock.node.write_later(LineState::Low, PropogationDelay(2000), &mut c);
    
    //not_s.node.write_later(LineState::High, PropogationDelay(1000), &mut c);
    //not_r.node.write_later(LineState::Low, PropogationDelay(2000), &mut c);
    
    loop {
        let more = c.play();
        if !more { break; }
    }
    
    println!("q = {:?}, ~q = {:?}", d.q.read(&c), d.not_q.read(&c));
    println!("r = {:?}", r.read_u64(&c));
    
    //println!("settled at t={}. out = {:?}", c.current_tick, overall_output.node.read(&c));
}