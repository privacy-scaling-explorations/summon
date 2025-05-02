use std::{cmp::max, collections::BTreeMap};

use crate::{binary_op::BinaryOp, unary_op::UnaryOp};
use bristol_circuit::{BristolCircuit, CircuitInfo, ConstantInfo, Gate as BristolGate, IOInfo};
use serde_json::json;
use summon_common::InputDescriptor;

use crate::bristol_op_strings::{to_bristol_binary_op, to_bristol_unary_op};

#[derive(Debug)]
pub struct Circuit {
  pub size: usize,
  pub constants: BTreeMap<usize, serde_json::Value>, // wire_id -> value
  pub inputs: BTreeMap<String, CircuitInput>,
  pub outputs: BTreeMap<String, usize>,
  pub mpc_settings: MpcSettings,
  pub gates: Vec<Gate>,
}

#[derive(Debug)]
pub struct CircuitInput {
  pub wire_id: usize,
  pub type_json: serde_json::Value, // TODO: rename to type_
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MpcParticipantSettings {
  pub name: String,
  pub inputs: Vec<String>,
  pub outputs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MpcSettings(Vec<MpcParticipantSettings>);

impl MpcSettings {
  pub fn from_io(
    parties: &[String],
    input_descriptors: &[InputDescriptor],
    outputs: Vec<String>,
  ) -> Self {
    let mut participants = parties
      .iter()
      .map(|name| MpcParticipantSettings {
        name: name.clone(),
        inputs: vec![],
        outputs: vec![],
      })
      .collect::<Vec<_>>();

    for desc in input_descriptors {
      let Some(participant) = participants.iter_mut().find(|p| p.name == desc.from) else {
        panic!("Participant {} not found", desc.from);
      };

      participant.inputs.push(desc.name.clone());
    }

    for participant in &mut participants {
      participant.outputs = outputs.clone();
    }

    Self(participants)
  }
}

#[derive(Debug)]
pub enum Gate {
  Unary {
    op: UnaryOp,
    input: usize,
    output: usize,
  },
  Binary {
    op: BinaryOp,
    left: usize,
    right: usize,
    output: usize,
  },
}

impl Circuit {
  pub fn eval<N: CircuitNumber>(&self, inputs: &BTreeMap<String, N>) -> BTreeMap<String, N> {
    let mut wire_values = vec![N::zero(); self.size];

    for (
      name,
      CircuitInput {
        wire_id,
        type_json: _,
      },
    ) in &self.inputs
    {
      let value = inputs.get(name).expect("Missing input");
      wire_values[*wire_id] = value.clone();
    }

    for (wire_id, value) in &self.constants {
      wire_values[*wire_id] = N::from_json(value);
    }

    for gate in &self.gates {
      match gate {
        Gate::Unary { op, input, output } => {
          wire_values[*output] = N::unary_op(*op, &wire_values[*input])
        }
        Gate::Binary {
          op,
          left,
          right,
          output,
        } => wire_values[*output] = N::binary_op(*op, &wire_values[*left], &wire_values[*right]),
      }
    }

    let mut res = BTreeMap::<String, N>::new();

    for (name, wire_id) in &self.outputs {
      res.insert(name.clone(), wire_values[*wire_id].clone());
    }

    res
  }

  pub fn depth(&self) -> usize {
    let mut wire_depths = vec![0usize; self.size];

    for gate in &self.gates {
      match gate {
        Gate::Unary {
          op: _,
          input,
          output,
        } => wire_depths[*output] = 1 + wire_depths[*input],
        Gate::Binary {
          op: _,
          left,
          right,
          output,
        } => wire_depths[*output] = 1 + max(wire_depths[*left], wire_depths[*right]),
      }
    }

    let max_depth = wire_depths.iter().fold(0, |a, b| max(a, *b));

    max_depth
  }

  pub fn to_bristol(&self) -> BristolCircuit {
    let mut bristol_gates = Vec::<BristolGate>::new();

    for gate in &self.gates {
      bristol_gates.push(match gate {
        Gate::Unary { op, input, output } => BristolGate {
          inputs: vec![*input],
          outputs: vec![*output],
          op: to_bristol_unary_op(*op),
        },
        Gate::Binary {
          op,
          left,
          right,
          output,
        } => BristolGate {
          inputs: vec![*left, *right],
          outputs: vec![*output],
          op: to_bristol_binary_op(*op),
        },
      });
    }

    let constants: Vec<ConstantInfo> = self
      .constants
      .iter()
      .map(|(id, value)| ConstantInfo {
        name: format!("constant_{}", value),
        type_: if value.is_boolean() {
          json!("bool")
        } else if value.is_number() {
          json!("number")
        } else {
          panic!("Unsupported constant type")
        },
        value: value.clone(),
        address: *id,
        width: 1,
      })
      .collect();

    let mut inputs: Vec<IOInfo> = self
      .inputs
      .iter()
      .map(|(name, CircuitInput { wire_id, type_json })| IOInfo {
        name: name.clone(),
        type_: type_json.clone(),
        address: *wire_id,
        width: 1,
      })
      .collect();

    inputs.sort_by_key(|io| io.address);

    let mut outputs: Vec<IOInfo> = self
      .outputs
      .iter()
      .map(|(name, id)| IOInfo {
        name: name.clone(),
        type_: json!("number"),
        address: *id,
        width: 1,
      })
      .collect();

    outputs.sort_by_key(|io| io.address);

    BristolCircuit {
      wire_count: self.size,
      info: CircuitInfo {
        constants,
        inputs,
        outputs,
      },
      gates: bristol_gates,
    }
  }
}

pub trait CircuitNumber: Clone {
  fn zero() -> Self;
  fn from_json(x: &serde_json::Value) -> Self;
  fn unary_op(op: UnaryOp, input: &Self) -> Self;
  fn binary_op(op: BinaryOp, left: &Self, right: &Self) -> Self;
}

impl CircuitNumber for usize {
  fn zero() -> Self {
    0
  }

  fn from_json(x: &serde_json::Value) -> Self {
    if let Some(x) = x.as_u64() {
      return x as usize;
    }

    if let Some(x) = x.as_bool() {
      return if x { 1 } else { 0 };
    }

    panic!("Couldn't convert to usize: {}", x);
  }

  // fn from_usize(x: usize) -> Self {
  //   x
  // }

  fn unary_op(op: UnaryOp, input: &Self) -> Self {
    let input = *input;

    match op {
      UnaryOp::Plus => input,
      UnaryOp::Minus => 0usize.wrapping_sub(input),
      UnaryOp::Not => (input == 0) as usize,
      UnaryOp::BitNot => !input,
    }
  }

  fn binary_op(op: BinaryOp, left: &Self, right: &Self) -> Self {
    let left = *left;
    let right = *right;

    match op {
      BinaryOp::Plus => left.wrapping_add(right),
      BinaryOp::Minus => left.wrapping_sub(right),
      BinaryOp::Mul => left.wrapping_mul(right),
      BinaryOp::Div => left / right,
      BinaryOp::Mod => left % right,
      BinaryOp::Exp => left.wrapping_pow(right as u32),
      BinaryOp::LooseEq => (left == right) as usize,
      BinaryOp::LooseNe => (left != right) as usize,
      BinaryOp::Eq => (left == right) as usize,
      BinaryOp::Ne => (left != right) as usize,
      BinaryOp::And => (left != 0 && right != 0) as usize,
      BinaryOp::Or => (left != 0 || right != 0) as usize,
      BinaryOp::Less => (left < right) as usize,
      BinaryOp::LessEq => (left <= right) as usize,
      BinaryOp::Greater => (left > right) as usize,
      BinaryOp::GreaterEq => (left >= right) as usize,
      BinaryOp::BitAnd => left & right,
      BinaryOp::BitOr => left | right,
      BinaryOp::BitXor => left ^ right,
      BinaryOp::LeftShift => left.wrapping_shl(right as u32),
      BinaryOp::RightShift => left.wrapping_shr(right as u32),
      BinaryOp::RightShiftUnsigned => left.wrapping_shr(right as u32),
    }
  }
}
