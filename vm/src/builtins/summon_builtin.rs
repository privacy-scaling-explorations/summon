use std::fmt;
use std::rc::Rc;

use crate::circuit_signal::CircuitSignal;
use crate::native_function::{native_fn, NativeFunction};
use crate::val_dynamic_downcast::val_dynamic_downcast;
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, ToVal, Val};

use super::builtin_object::BuiltinObject;
use super::type_error_builtin::ToTypeError;

pub struct SummonBuiltin {}

impl BuiltinObject for SummonBuiltin {
  fn bo_name() -> &'static str {
    "summon"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "isSignal" => IS_SIGNAL.to_val(),
      "number" => NUMBER.to_val(),

      _ => Val::Undefined,
    }
  }

  fn bo_load_function() -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn bo_as_class_data() -> Option<Rc<VsClass>> {
    None
  }
}

impl fmt::Display for SummonBuiltin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[object summon]")
  }
}

static IS_SIGNAL: NativeFunction = native_fn(|_this, params| {
  let Some(x) = params.first() else {
    return Ok(false.to_val());
  };

  Ok(val_dynamic_downcast::<CircuitSignal>(x).is_some().to_val())
});

static NUMBER: NativeFunction = native_fn(|_this, params| {
  if !params.is_empty() {
    return Err("Unexpected arguments".to_type_error());
  }

  Ok(Val::make_object(&[
    ("about", "summon runtime type".to_val()),
    ("json", "number".to_val()),
  ]))
});
