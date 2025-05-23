use std::{cell::RefCell, rc::Rc};

use crate::{
  binary_op::BinaryOp,
  operations::op_or,
  type_error_builtin::ToTypeError,
  unary_op::UnaryOp,
  vs_value::{ToDynamicVal, ToVal, Val},
  LoadFunctionResult, ValTrait,
};
use num_bigint::BigInt;

use crate::vs_value::VsType;
use crate::{id_generator::IdGenerator, val_dynamic_downcast::val_dynamic_downcast};

#[derive(Clone)]
pub enum CircuitSignalData {
  Input,
  UnaryOp(UnaryOp, Val),
  BinaryOp(BinaryOp, Val, Val),
}

#[derive(Clone)]
pub struct CircuitSignal {
  pub type_: VsType,
  pub data: CircuitSignalData,
  pub id: usize,
  pub id_generator: Rc<RefCell<IdGenerator>>,
}

impl CircuitSignal {
  pub fn new(
    id_generator: &Rc<RefCell<IdGenerator>>,
    type_: Option<VsType>,
    data: CircuitSignalData,
  ) -> Self {
    CircuitSignal {
      type_: type_.unwrap_or_else(|| typeof_(&data)),
      data,
      id: id_generator.borrow_mut().gen(),
      id_generator: id_generator.clone(),
    }
  }
}

impl ValTrait for CircuitSignal {
  fn to_number(&self) -> f64 {
    f64::NAN
  }

  fn typeof_(&self) -> VsType {
    self.type_
  }

  fn to_index(&self) -> Option<usize> {
    panic!("Not implemented: using CircuitSignal as index")
  }

  fn is_primitive(&self) -> bool {
    false
  }

  fn is_truthy(&self) -> bool {
    panic!("Not implemented: truthiness of CircuitSignal")
  }

  fn is_nullish(&self) -> bool {
    false
  }

  fn bind(&self, _params: Vec<Val>) -> Option<Val> {
    None
  }

  fn as_bigint_data(&self) -> Option<BigInt> {
    None
  }

  fn as_array_data(&self) -> Option<Rc<crate::vs_array::VsArray>> {
    None
  }

  fn as_class_data(&self) -> Option<Rc<crate::vs_class::VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, _key: &Val) -> Result<Val, Val> {
    Ok(Val::Undefined)
  }

  fn has(&self, _key: &Val) -> Option<bool> {
    None
  }

  fn submov(&mut self, _key: &Val, _value: Val) -> Result<(), Val> {
    Err("Cannot assign to subscript of CircuitSignal".to_type_error())
  }

  fn override_binary_op(&self, op: BinaryOp, left: &Val, right: &Val) -> Option<Result<Val, Val>> {
    if left.typeof_() != VsType::Number && left.typeof_() != VsType::Bool {
      return None;
    }

    if right.typeof_() != VsType::Number && right.typeof_() != VsType::Bool {
      return None;
    }

    match op {
      BinaryOp::Plus => {
        if let Val::Number(left) = left {
          if *left == 0.0 {
            return Some(Ok(right.clone()));
          }
        }

        if let Val::Number(right) = right {
          if *right == 0.0 {
            return Some(Ok(left.clone()));
          }
        }
      }
      BinaryOp::Mul => {
        if let Val::Number(left) = left {
          if *left == 1.0 {
            return Some(Ok(right.clone()));
          }

          if *left == 0.0 {
            return Some(Ok(0f64.to_val()));
          }
        }

        if let Val::Number(right) = right {
          if *right == 1.0 {
            return Some(Ok(left.clone()));
          }

          if *right == 0.0 {
            return Some(Ok(0f64.to_val()));
          }
        }
      }
      BinaryOp::Or => {
        if related_by_negation(left, right) {
          return Some(Ok(true.to_val()));
        }

        if let Some((y, a, b)) = common_and(left, right) {
          // (y && a) || (y && b) => y && (a || b)

          return Some(op_or(a, b).map(|a_or_b| {
            CircuitSignal::new(
              &y.id_generator,
              None,
              CircuitSignalData::BinaryOp(BinaryOp::And, y.clone().to_dynamic_val(), a_or_b),
            )
            .to_dynamic_val()
          }));
        }

        // TODO: (y && a) || (y && b) => y && (a || b)
        //       (y && a) || (y && !a) => y && (a || !a) => y

        if left.typeof_() == VsType::Bool && right.typeof_() == VsType::Bool {
          match left {
            Val::Bool(true) => return Some(Ok(true.to_val())),
            Val::Bool(false) => return Some(Ok(right.clone())),
            _ => {}
          };

          match right {
            Val::Bool(true) => return Some(Ok(true.to_val())),
            Val::Bool(false) => return Some(Ok(left.clone())),
            _ => {}
          }
        }
      }
      BinaryOp::And => {
        if left.typeof_() == VsType::Bool && right.typeof_() == VsType::Bool {
          match left {
            Val::Bool(true) => return Some(Ok(right.clone())),
            Val::Bool(false) => return Some(Ok(false.to_val())),
            _ => {}
          };

          match right {
            Val::Bool(true) => return Some(Ok(left.clone())),
            Val::Bool(false) => return Some(Ok(false.to_val())),
            _ => {}
          }
        }
      }
      _ => {}
    }

    Some(Ok(
      CircuitSignal::new(
        &self.id_generator,
        None,
        CircuitSignalData::BinaryOp(op, left.clone(), right.clone()),
      )
      .to_dynamic_val(),
    ))
  }

  fn override_unary_op(&self, op: UnaryOp, input: &Val) -> Option<Result<Val, Val>> {
    if op == UnaryOp::Plus && val_dynamic_downcast::<CircuitSignal>(input).is_some() {
      return Some(Ok(
        CircuitSignal::new(&self.id_generator, Some(VsType::Number), self.data.clone())
          .to_dynamic_val(),
      ));
    }

    Some(Ok(
      CircuitSignal::new(
        &self.id_generator,
        None,
        CircuitSignalData::UnaryOp(op, input.clone()),
      )
      .to_dynamic_val(),
    ))
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[CircuitSignal]")
  }

  fn codify(&self) -> String {
    "[CircuitSignal]".into()
  }
}

fn related_by_negation(left: &Val, right: &Val) -> bool {
  if let Some((left, right)) = both_circuit_signals(left, right) {
    if let CircuitSignalData::UnaryOp(UnaryOp::Not, not_left) = &left.data {
      if let Some(not_left) = val_dynamic_downcast::<CircuitSignal>(not_left) {
        if not_left.id == right.id {
          return true;
        }
      }
    }

    if let CircuitSignalData::UnaryOp(UnaryOp::Not, not_right) = &right.data {
      if let Some(not_right) = val_dynamic_downcast::<CircuitSignal>(not_right) {
        if not_right.id == left.id {
          return true;
        }
      }
    }
  }

  false
}

fn common_and<'a>(left: &'a Val, right: &'a Val) -> Option<(&'a CircuitSignal, &'a Val, &'a Val)> {
  if let Some((left, right)) = both_circuit_signals(left, right) {
    if let (
      CircuitSignalData::BinaryOp(BinaryOp::And, left_lhs, left_rhs),
      CircuitSignalData::BinaryOp(BinaryOp::And, right_lhs, right_rhs),
    ) = (&left.data, &right.data)
    {
      if let Some((left_lhs, right_lhs)) = both_circuit_signals(left_lhs, right_lhs) {
        if left_lhs.id == right_lhs.id {
          return Some((left_lhs, left_rhs, right_rhs));
        }
      }
    }
  }

  None
}

fn both_circuit_signals<'a>(
  left: &'a Val,
  right: &'a Val,
) -> Option<(&'a CircuitSignal, &'a CircuitSignal)> {
  if let (Some(left), Some(right)) = (
    val_dynamic_downcast::<CircuitSignal>(left),
    val_dynamic_downcast::<CircuitSignal>(right),
  ) {
    Some((left, right))
  } else {
    None
  }
}

impl std::fmt::Display for CircuitSignal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[CircuitSignal]")
  }
}

fn typeof_(data: &CircuitSignalData) -> VsType {
  match data {
    CircuitSignalData::Input { .. } => VsType::Number,
    CircuitSignalData::UnaryOp(op, _) => match op {
      UnaryOp::Plus => VsType::Number,
      UnaryOp::Minus => VsType::Number,
      UnaryOp::Not => VsType::Bool,
      UnaryOp::BitNot => VsType::Number,
    },
    CircuitSignalData::BinaryOp(op, left, right) => match op {
      BinaryOp::Plus => VsType::Number,
      BinaryOp::Minus => VsType::Number,
      BinaryOp::Mul => VsType::Number,
      BinaryOp::Div => VsType::Number,
      BinaryOp::Mod => VsType::Number,
      BinaryOp::Exp => VsType::Number,
      BinaryOp::LooseEq => VsType::Bool,
      BinaryOp::LooseNe => VsType::Bool,
      BinaryOp::Eq => VsType::Bool,
      BinaryOp::Ne => VsType::Bool,
      BinaryOp::And | BinaryOp::Or => match (left.typeof_(), right.typeof_()) {
        (VsType::Number, VsType::Number) => VsType::Number,
        (VsType::Bool, VsType::Bool) => VsType::Bool,
        (left, right) => panic!("Incompatible types {} {}", left, right),
      },
      BinaryOp::Less => VsType::Bool,
      BinaryOp::LessEq => VsType::Bool,
      BinaryOp::Greater => VsType::Bool,
      BinaryOp::GreaterEq => VsType::Bool,
      BinaryOp::BitAnd => VsType::Number,
      BinaryOp::BitOr => VsType::Number,
      BinaryOp::BitXor => VsType::Number,
      BinaryOp::LeftShift => VsType::Number,
      BinaryOp::RightShift => VsType::Number,
      BinaryOp::RightShiftUnsigned => VsType::Number,
    },
  }
}
