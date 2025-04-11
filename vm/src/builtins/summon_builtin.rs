use std::fmt;
use std::rc::Rc;

use crate::native_function::{native_fn, NativeFunction};
use crate::vs_class::VsClass;
use crate::vs_value::{LoadFunctionResult, ToVal, Val};

use super::builtin_object::BuiltinObject;

pub struct SummonBuiltin {}

impl BuiltinObject for SummonBuiltin {
  fn bo_name() -> &'static str {
    "summon"
  }

  fn bo_sub(key: &str) -> Val {
    match key {
      "isSignal" => IS_SIGNAL.to_val(),

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

static IS_SIGNAL: NativeFunction = native_fn(|_this, params| Ok(Val::Bool(true)));
