use std::collections::HashMap;
use crate::model::{Circuit, ComponentID, Components, TerminalID};
use nalgebra::{DMatrix, DVector};
use crate::disjoint_set::DisjointSet;

pub struct Simulator {
    circuit: Circuit,
    n: usize,
    component_id_to_vgenerator_id: HashMap<ComponentID, usize>,
    nodes: Vec<Vec<TerminalID>>,
    terminal_to_node: HashMap<TerminalID, usize>,

    vgenerators: Vec<ComponentID>,
}

impl Simulator {
    /**
    Creates a new simulator.

    The simulator tries to solve M * X = Y, where M is the matrix of the circuit, X is the unknowns
    and Y is the result.

    ## Result (Y):
    transpose(Y) = [I_1, I_2, ..., I_n, V_g0, V_g1, ..., V_gm]

    Where:
    - Where I_i is the intensity that goes through the node i. I_i = 0 as per the Kirchhoff's law.
    - V_sj is the voltage of the "voltage generator" j.

    ## Unknowns (X):
    transpose(X) = [V_1, V_2, ..., V_n, I_g0, I_g1, ..., I_gm]

    Where:
    - V_i is the voltage of the node i. NOTE: By convention, V_0 = 0.
    - I_sj is the intensity that goes through the voltage generator j.
    */
    pub fn new(circuit: Circuit) -> Self {
        let terminal_ids: Vec<TerminalID> = circuit.terminal_edges
            .iter()
            .flat_map(|(left, right)| [*left, *right])
            .collect();

        let nodes = {
            let mut terminal_disjoint_set = DisjointSet::new(terminal_ids);

            for (left, right) in &circuit.terminal_edges {
                terminal_disjoint_set.merge(left.clone(), right.clone());
            }
            terminal_disjoint_set.into_terminal_groups()
        };

        let vgenerators: Vec<ComponentID> = circuit.components
            .iter()
            .enumerate()
            .filter_map(|(component_id, component)| {
                match component {
                    Components::VoltageGenerator(_) => Some(ComponentID(component_id)),
                    _ => None,
                }
            })
            .collect();



        let terminal_id_to_node_id = {
            let mut m = HashMap::new();
            for (node_id, node) in nodes.iter().enumerate() {
                for terminal_id in node {
                    m.insert(terminal_id.clone(), node_id);
                }
            }
            m
        };

        let component_id_to_vgenerator_id = {
            let mut m = HashMap::new();
            for (generator_id, component_id) in vgenerators.iter().enumerate() {
                m.insert(*component_id, generator_id);
            }
            m
        };

        let n = nodes.len() - 1 + vgenerators.len();

        Self { circuit, component_id_to_vgenerator_id, nodes, terminal_to_node: terminal_id_to_node_id, n, vgenerators }
    }

    pub fn simulate(&self) {
        let mat = self.get_matrix();
        println!("Matrix: {}", mat);

        let result = self.get_result_vector();
        println!("Result: {:?}", result);

        // let cloned_mats = vec![mat.clone(); 10_000];
        // let start = std::time::Instant::now();
        // for cloned_mat in cloned_mats {
        //     cloned_mat.lu().solve(&result).unwrap();
        // }
        // println!("Elapsed: {:?}", start.elapsed());

        let unknowns = mat.lu().solve(&result).unwrap();
        println!("Unknowns: {:?}", unknowns);

        for component_id in 0..self.circuit.components.len() {
            let component = &self.circuit.components[component_id];
            let input_terminal_id = TerminalID::new(component_id, 0);
            let output_terminal_id = TerminalID::new(component_id, 1);

            let node_input = self.get_node_id_from_terminal_id(&input_terminal_id);
            let node_output = self.get_node_id_from_terminal_id(&output_terminal_id);

            let v_input = if node_input >= 1  {
                unknowns[node_input - 1]
            } else {
                0.0
            };
            let v_output = if node_output >= 1 {
                unknowns[node_output - 1]
            } else {
                0.0
            };

            let v = v_output - v_input;

            match component {
                Components::Resistor(_) => {
                    println!("Resistor {}: {}V", &component_id, v);
                }
                Components::VoltageGenerator(_) => {
                    println!("Voltage Generator {}: {}V", &component_id, v);
                }
            }
        }
    }

    /** Returns the matrix (M) of the equation (M * X = Y). */
    pub fn get_matrix(&self) -> DMatrix<f64> {
        let mut rows = Vec::with_capacity(self.n);
        for node_id in 1..self.nodes.len() {
            let node_intensity = self.get_node_intensity(node_id);
            rows.push(node_intensity.transpose());
        }

        for vgenerator_id in 0..self.vgenerators.len() {
            let vgenerator_intensity = self.get_vgenerator_voltage(vgenerator_id);
            rows.push(vgenerator_intensity.transpose());
        }

        DMatrix::from_rows(&rows)
    }


    /** Returns the intensity that goes through a certain node as a vector of the dimensions. */
    fn get_node_intensity(&self, node_id: usize) -> DVector<f64> {
        let mut result = DVector::zeros(self.n);

        for terminal_id in &self.nodes[node_id] {
            let intensity = self.get_component_intensity_vector(*terminal_id);
            result += intensity;
        }

        result
    }

    /** Returns the intensity that goes through a certain component as a vector of the dimensions. */
    fn get_component_intensity_vector(&self, output_terminal_id: TerminalID) -> DVector<f64> {
        let component = &self.circuit.components[output_terminal_id.component_id.0];
        let input_terminal_id = Self::get_other_terminal(&output_terminal_id);

        match component {
            Components::Resistor(resistance) => {
                let node_output = self.get_node_id_from_terminal_id(&output_terminal_id);
                let node_input = self.get_node_id_from_terminal_id(&input_terminal_id);

                let v_output = self.unknown_node_voltage(node_output);
                let v_input = self.unknown_node_voltage(node_input);

                (v_output - v_input) / *resistance
            }
            Components::VoltageGenerator(_) => {
                let generator_id = self.get_vgenerator_id_from_component_id(&output_terminal_id.component_id);
                let intensity = self.unknown_vgenerator_intensity(generator_id);

                // This intensity is directed from 0->1. So if we want the intensity on the other
                // terminal, we need to invert it.
                if output_terminal_id.idx == 1 {
                    intensity
                } else {
                    -intensity
                }
            }
        }
    }

    fn get_vgenerator_voltage(&self, vgenerator_id: usize) -> DVector<f64> {
        let component_id = self.get_component_id_from_vgenerator_id(vgenerator_id);

        let terminal_input = TerminalID::new(component_id.0, 0);
        let terminal_output = TerminalID::new(component_id.0, 1);

        let node_input = self.get_node_id_from_terminal_id(&terminal_input);
        let node_output = self.get_node_id_from_terminal_id(&terminal_output);

        let v_input = self.unknown_node_voltage(node_input);
        let v_output = self.unknown_node_voltage(node_output);

        v_output - v_input
    }


    /** Returns the result (Y) of the matrix equation (M * X = Y). */
    fn get_result_vector(&self) -> DVector<f64> {
        let mut result = DVector::zeros(self.n);

        for node_id in 0..(self.nodes.len() - 1) {
            result[node_id] = 0.0; // Sum of all currents in the node.
        }

        for (vgenerator_id, generator) in self.vgenerators.iter().enumerate() {
            let component = &self.circuit.components[generator.0];
            let voltage = match component {
                Components::VoltageGenerator(voltage) => voltage,
                _ => panic!("Voltage generator expected"),
            };
            result[self.nodes.len() - 1 + vgenerator_id] = *voltage;
        }

        result
    }


    /** Assuming the terminal is a bipolar terminal, return the other terminal of the same component. */
    fn get_other_terminal(terminal_id: &TerminalID) -> TerminalID {
        if terminal_id.idx == 0 {
            TerminalID::new(terminal_id.component_id.0, 1)
        } else if terminal_id.idx == 1 {
            TerminalID::new(terminal_id.component_id.0, 0)
        } else {
            panic!("Invalid terminal index")
        }
    }

    /** Returns the Node ID from which a terminal is connected. */
    fn get_node_id_from_terminal_id(&self, terminal_id: &TerminalID) -> usize {
        self.terminal_to_node[terminal_id]
    }

    /** Return the voltage generator ID from the component ID. */
    fn get_vgenerator_id_from_component_id(&self, component_id: &ComponentID) -> usize {
        self.component_id_to_vgenerator_id[component_id]
    }

    fn get_component_id_from_vgenerator_id(&self, vgenerator_id: usize) -> ComponentID {
        self.vgenerators[vgenerator_id]
    }


    /** Represents the voltage of a node as a unit vector. */
    fn unknown_node_voltage(&self, node_id: usize) -> DVector<f64> {
        if node_id == 0 {
            // By convention, the node id=0 is the ground node.
            return DVector::zeros(self.n);
        }

        let idx = node_id - 1;

        let mut result = DVector::zeros(self.n);
        result[idx] = 1.0;
        result
    }

    /** Represents the intensity that go through a voltage generator as a unit vector. */
    fn unknown_vgenerator_intensity(&self, vgenerator_id: usize) -> DVector<f64> {
        let idx = self.nodes.len() - 1 + vgenerator_id;

        let mut result = DVector::zeros(self.n);
        result[idx] = 1.0;
        result
    }
}