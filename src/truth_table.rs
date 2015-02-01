
use sim::{NodeCreator, NodeIndex, STANDARD_DELAY, NodeCollection, LineState};
use pin::Pin;

pub fn check_truth_table<F>(f: F, cases: &[(&[u8],&[u8])]) 
    where F: FnOnce(&mut NodeCreator) -> (Vec<NodeIndex>, Vec<NodeIndex>)
{
    let mut c = NodeCollection::new();
    let mut creator = NodeCreator::new(&c);
    
    let (inputs, outputs) = f(&mut creator);
    let input_pins : Vec<Pin> = inputs.iter().map(|input| {
        let p = Pin::new(&mut creator);
        creator.link(*input, p.node, STANDARD_DELAY);
        p
    }).collect();
    
    c.absorb(creator);
    
    for (case_number, &(input_values, output_values)) in cases.iter().enumerate() {
        assert!(input_values.len() == input_pins.len());
        assert!(output_values.len() == outputs.len());
        for (input_pin, input_value) in input_pins.iter().zip(input_values.iter()) {
            assert!(*input_value==0 || *input_value==1);
            input_pin.node.write(if *input_value==1 { LineState::High } else { LineState::Low }, &mut c);
        }
        
        
        loop {
            let more = c.play();
            if !more { break; }
        }
        
        let actual : Vec<u8> = outputs.iter().map(|output_node| {
            match output_node.read(&c) {
                LineState::Low => 0,
                LineState::High => 1,
                _ => 2
            }
        }).collect();
        
        let expected = output_values.to_vec();
        assert!(actual==expected, "Case #{}. For inputs {:?}, expected and actual:\n{:?}\n{:?}", case_number+1, input_values.to_vec(), expected, actual);
    }
} 
