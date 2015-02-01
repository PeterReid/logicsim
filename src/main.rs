#![feature(core, rustc_private)]

extern crate arena;

mod sim;
mod storage;
mod logic_gates;
mod pin;
mod nand;

use arena::Arena;

use pin::Pin;

use sim::{LineState, NodeIndex, NodeCreator, Element, NodeCollection, PropogationDelay, STANDARD_DELAY};
use storage::{Register};

fn main() {
    let mut arena: Arena = Arena::new();
    
    let mut c = NodeCollection::new();
    let mut creator = NodeCreator::new(&c);
    let power = Pin::new(&mut creator, &arena);
    let ground = Pin::new(&mut creator, &arena);
    let overall_output = Pin::new(&mut creator, &arena);
    let clock = Pin::new(&mut creator, &arena);
    let data = Pin::new(&mut creator, &arena);
    
    //let not_s = arena.alloc(|| { Pin::new(&mut creator) });
    //let not_r = arena.alloc(|| { Pin::new(&mut creator) });
    
    //c.add_element(not_s);
    //c.add_element(not_r);
    
    //let sr1 = NotSRLatch::new(&mut creator, &arena);
    //creator.link(not_s.node, sr1.not_s, STANDARD_DELAY);
    //creator.link(not_r.node, sr1.not_r, STANDARD_DELAY);
    
    
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
    
    println!("r = {:?}", r.read_u64(&c));
    
    //println!("settled at t={}. out = {:?}", c.current_tick, overall_output.node.read(&c));
}