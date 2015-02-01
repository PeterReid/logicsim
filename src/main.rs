#![feature(core, rustc_private)]

extern crate arena;

mod sim;
mod storage;
mod logic_gates;
mod pin;
mod nand;
mod adder;
mod truth_table;

use pin::Pin;

use sim::{LineState, NodeIndex, NodeCreator, Element, NodeCollection, PropogationDelay, STANDARD_DELAY};
use storage::{Register};

fn main() {
    let mut c = NodeCollection::new();
    let mut creator = NodeCreator::new(&c);
    let power = Pin::new(&mut creator);
    let ground = Pin::new(&mut creator);
    let overall_output = Pin::new(&mut creator);
    let clock = Pin::new(&mut creator);
    let data = Pin::new(&mut creator);
    
    
    let r = Register::new(&mut creator, 8);
    creator.link(clock.node, r.clock, STANDARD_DELAY);
    creator.link(power.node, r.bits[0].data, STANDARD_DELAY);
    
    c.absorb(creator);
    
    power.node.write(LineState::High, &mut c);
    ground.node.write(LineState::Low, &mut c);
    data.node.write(LineState::Low, &mut c);
    clock.node.write(LineState::Low, &mut c);
    
    clock.node.write_later(LineState::High, PropogationDelay(1000), &mut c);
    clock.node.write_later(LineState::Low, PropogationDelay(2000), &mut c);
    
    loop {
        let more = c.play();
        if !more { break; }
    }
    
    println!("r = {:?}", r.read_u64(&c));
    
    println!("exited at t={}", c.current_tick);
}