#[derive(Debug)]
pub struct Circuit {
    pub components: Vec<Components>,
    pub terminal_edges: Vec<(TerminalID, TerminalID)>,
}

#[derive(Debug)]
pub enum Components {
    Resistor(f64),
    VoltageGenerator(f64),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ComponentID(pub usize);


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TerminalID {
    pub component_id: ComponentID,
    pub idx: usize,
}

impl TerminalID {
    pub fn new(component_id: usize, terminal_id: usize) -> Self {
        let component_id = ComponentID(component_id);
        Self {
            component_id,
            idx: terminal_id,
        }
    }
}

