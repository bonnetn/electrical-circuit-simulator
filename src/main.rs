use crate::model::{Circuit, Components, TerminalID};
use crate::simulator::Simulator;

mod model;
mod disjoint_set;
mod simulator;

fn main() {
    let circuit = Circuit {
        components: vec![
            Components::VoltageGenerator(10.0),
            Components::Resistor(2.0),
            Components::Resistor(4.0),
            Components::Resistor(3.0),
        ],
        terminal_edges: vec![
            (TerminalID::new(0, 1), TerminalID::new(1, 0)),
            (TerminalID::new(1, 1), TerminalID::new(2, 0)),
            (TerminalID::new(2, 1), TerminalID::new(0, 0)),
            (TerminalID::new(1, 1), TerminalID::new(3, 0)),
            (TerminalID::new(3, 1), TerminalID::new(0, 0)),
        ],
    };

    let simulator = Simulator::new(circuit);
    simulator.simulate();
}



