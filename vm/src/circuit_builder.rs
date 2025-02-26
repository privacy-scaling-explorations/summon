use std::{
  collections::{BTreeMap, HashMap, HashSet},
  mem::swap,
};

use crate::{vs_value::Val, ValTrait};
use num_traits::ToPrimitive;

use crate::{
  circuit::Gate,
  circuit_signal::{CircuitSignal, CircuitSignalData},
};

#[derive(Default)]
pub struct CircuitBuilder {
  pub gates: Vec<Gate>,
  pub wire_count: usize,
  pub wires_included: HashMap<usize, usize>, // CircuitSignal.id -> wire_id
  pub constants: HashMap<usize, usize>,      // value -> wire_id
}

impl CircuitBuilder {
  pub fn include_inputs(&mut self, input_len: usize) {
    for i in 0..input_len {
      self.wires_included.insert(i, i);
    }

    self.wire_count = input_len;
  }

  pub fn include_outputs(&mut self, outputs: &Vec<Val>) -> Vec<usize> {
    for output in outputs {
      for dep in get_dependencies(output) {
        self.include_val(&dep);
      }
    }

    let mut output_ids = vec![];

    for output in outputs {
      output_ids.push(self.include_val(output));
    }

    output_ids
  }

  pub fn include_val(&mut self, val: &Val) -> usize {
    if let Some(signal) = as_circuit_signal(val) {
      return self.include_signal(signal);
    }

    self.include_val_shallow(val)
  }

  pub fn include_val_shallow(&mut self, val: &Val) -> usize {
    match val {
      Val::Bool(bool) => {
        let value = if *bool { 1usize } else { 0usize };

        if let Some(wire_id) = self.constants.get(&value) {
          return *wire_id;
        }

        let wire_id = self.wire_count;
        self.wire_count += 1;
        self.constants.insert(value, wire_id);

        wire_id
      }
      Val::Number(number) => {
        if *number != number.trunc() {
          panic!("Cannot use non-integer constant");
        }

        let value = if *number < 0.0 {
          usize::MAX - ((-number).to_usize().unwrap() - 1)
        } else {
          number.to_usize().unwrap()
        };

        if let Some(wire_id) = self.constants.get(&value) {
          return *wire_id;
        }

        let wire_id = self.wire_count;
        self.wire_count += 1;
        self.constants.insert(value, wire_id);

        wire_id
      }
      Val::Dynamic(dyn_val) => {
        if let Some(signal) = dyn_val.as_any().downcast_ref::<CircuitSignal>() {
          if let Some(wire_id) = self.wires_included.get(&signal.id) {
            return *wire_id;
          }
        }

        panic!("Can't include unrecognized type ({}) 1", val.codify());
      }
      _ => panic!("Can't include unrecognized type ({}) 2", val.codify()),
    }
  }

  pub fn include_signal_shallow(
    &mut self,
    signal: &CircuitSignal,
    dependent_ids: Vec<usize>,
  ) -> usize {
    let wire_id = self.wire_count;
    self.wire_count += 1;

    let gate = match &signal.data {
      CircuitSignalData::Input => panic!("Input should have been included earlier"),
      CircuitSignalData::UnaryOp(op, _) => Gate::Unary {
        op: *op,
        input: dependent_ids[0],
        output: wire_id,
      },
      CircuitSignalData::BinaryOp(op, _, _) => Gate::Binary {
        op: *op,
        left: dependent_ids[0],
        right: dependent_ids[1],
        output: wire_id,
      },
    };

    self.gates.push(gate);

    self.wires_included.insert(signal.id, wire_id);

    wire_id
  }

  pub fn include_signal(&mut self, signal: &CircuitSignal) -> usize {
    if let Some(wire_id) = self.wires_included.get(&signal.id) {
      return *wire_id;
    }

    let mut signals_to_process = vec![signal.clone()];
    let mut signal_id_to_parent_signals = HashMap::<usize, Vec<CircuitSignal>>::new();
    let mut signal_id_to_dep_ids = HashMap::<usize, HashSet<usize>>::new();
    let mut id_to_leaf_signal = BTreeMap::<usize, CircuitSignal>::new();

    while let Some(signal) = signals_to_process.pop() {
      let mut dep_ids = HashSet::<usize>::new();

      for dep in get_signal_dependencies(&signal) {
        let Some(dep_signal) = as_circuit_signal(&dep) else {
          continue;
        };

        if self.wires_included.contains_key(&dep_signal.id) {
          continue;
        }

        dep_ids.insert(dep_signal.id);

        signal_id_to_parent_signals
          .entry(dep_signal.id)
          .or_default()
          .push(signal.clone());

        signals_to_process.push(dep_signal.clone());
      }

      if dep_ids.is_empty() {
        id_to_leaf_signal.insert(signal.id, signal.clone());
      } else {
        signal_id_to_dep_ids.insert(signal.id, dep_ids);
      }
    }

    let mut next_id_to_leaf_signal = BTreeMap::<usize, CircuitSignal>::new();

    loop {
      for (_, leaf_signal) in id_to_leaf_signal.iter() {
        let dependent_ids = get_signal_dependencies(leaf_signal)
          .iter()
          .map(|dep| self.include_val_shallow(dep))
          .collect::<Vec<usize>>();

        self.include_signal_shallow(signal, dependent_ids);

        for parent_signal in signal_id_to_parent_signals
          .get(&leaf_signal.id)
          .unwrap_or(&vec![])
        {
          let dep_ids = signal_id_to_dep_ids.get_mut(&parent_signal.id).unwrap();
          dep_ids.remove(&leaf_signal.id);

          if dep_ids.is_empty() {
            next_id_to_leaf_signal.insert(parent_signal.id, parent_signal.clone());
          }
        }
      }

      if next_id_to_leaf_signal.is_empty() {
        break;
      }

      id_to_leaf_signal.clear();
      swap(&mut id_to_leaf_signal, &mut next_id_to_leaf_signal);
    }

    if let Some(wire_id) = self.wires_included.get(&signal.id) {
      return *wire_id;
    }

    panic!("Failed to include signal");
  }
}

fn get_dependencies(val: &Val) -> Vec<Val> {
  if let Val::Dynamic(val) = val {
    if let Some(circuit_number) = val.as_any().downcast_ref::<CircuitSignal>() {
      return get_signal_dependencies(circuit_number);
    }
  }

  vec![]
}

fn as_circuit_signal(val: &Val) -> Option<&CircuitSignal> {
  if let Val::Dynamic(val) = val {
    if let Some(signal) = val.as_any().downcast_ref::<CircuitSignal>() {
      return Some(signal);
    }
  }

  None
}

fn get_signal_dependencies(signal: &CircuitSignal) -> Vec<Val> {
  match &signal.data {
    CircuitSignalData::Input => vec![],
    CircuitSignalData::UnaryOp(_, input) => {
      vec![input.clone()]
    }
    CircuitSignalData::BinaryOp(_, left, right) => {
      vec![left.clone(), right.clone()]
    }
  }
}
