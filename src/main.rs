#![feature(core, rustc_private)]

extern crate arena;

mod sim;
mod storage;
mod logic_gates;
mod pin;
mod nand;
mod adder;
mod rom;
mod mux;
mod demux;

mod cpu0;

#[cfg(test)]
mod truth_table;

use pin::Pin;

use sim::{LineState, NodeIndex, NodeCreator, Element, NodeCollection, PropogationDelay, STANDARD_DELAY};
use storage::{Register};
use rom::{ConstantBit, ConstantBits};


fn main() {
    let mut c = NodeCollection::new();
    let mut creator = NodeCreator::new(&c);
    let power = ConstantBit::new(true, &mut creator);
    let ground = Pin::new(&mut creator);
    let overall_output = Pin::new(&mut creator);
    let clock = Pin::new(&mut creator);
    let data = Pin::new(&mut creator);
    
    let forty_two = ConstantBits::new(&ConstantBits::make_bits(42, 8)[], &mut creator);
    
    let r = Register::new(&mut creator, 8);
    creator.link(clock.node, r.clock, STANDARD_DELAY);
    //creator.link(power.node, r.bits[0].data, STANDARD_DELAY);
    
    creator.multilink(&forty_two.bits[], &r.inputs[], STANDARD_DELAY);
    
    
    c.absorb(creator);
    
    //power.node.write(LineState::High, &mut c);
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
    println!("power = {:?}", power.node.read(&c));
    println!("exited at t={}", c.current_tick);
}