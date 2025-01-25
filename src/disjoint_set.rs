use crate::model::TerminalID;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DisjointSet {
    elements: HashMap<TerminalID, TerminalID>,
}

impl DisjointSet {
    pub fn new(terminal_ids: Vec<TerminalID>) -> Self {
        let mut elements = HashMap::new();
        for edge in terminal_ids {
            elements.insert(edge.clone(), edge.clone());
        }

        Self { elements }
    }

    pub fn merge(&mut self, left: TerminalID, right: TerminalID) {
        let left = self.find(&left);
        let right = self.find(&right);
        self.elements.insert(left, right);
    }

    fn find(&self, terminal_id: &TerminalID) -> TerminalID {
        let mut id = terminal_id.clone();
        while self.elements.get(&id).expect("Terminal ID is not in the set") != &id {
            id = self.elements[&id].clone();
        }
        id
    }

    pub fn into_terminal_groups(self) -> Vec<Vec<TerminalID>> {
        let mut sets = HashMap::new();
        for (terminal_id, _) in &self.elements {
            let root = self.find(terminal_id);

            sets.entry(root)
                .or_insert_with(Vec::new)
                .push(terminal_id.clone());
        }

        sets.into_iter()
            .map(|(_, set)| set)
            .collect()
    }
}

