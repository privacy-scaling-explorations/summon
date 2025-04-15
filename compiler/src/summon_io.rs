use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

use num_bigint::BigInt;
use summon_vm::{
  error_builtin::ToError,
  internal_error_builtin::ToInternalError,
  native_function::{native_fn, NativeFunction},
  operations::op_triple_eq_impl,
  type_error_builtin::ToTypeError,
  val_dynamic_downcast::val_dynamic_downcast,
  vs_array::VsArray,
  vs_class::VsClass,
  vs_value::{ToVal, Val, VsType},
  LoadFunctionResult, ValTrait,
};

#[derive(Clone)]
pub struct SummonIO {
  pub data: Rc<RefCell<SummonIOData>>,
}

impl SummonIO {
  pub fn new(public_inputs: &HashMap<String, Val>) -> Self {
    Self {
      data: Rc::new(RefCell::new(SummonIOData {
        public_inputs: public_inputs.clone(),
        public_inputs_used: HashSet::new(),
        public_outputs: HashMap::new(),
      })),
    }
  }

  pub fn unused_public_inputs(&self) -> Vec<String> {
    let io_data = self.data.borrow();
    io_data
      .public_inputs
      .keys()
      .filter(|key| !io_data.public_inputs_used.contains(*key))
      .cloned()
      .collect()
  }
}

pub struct SummonIOData {
  pub public_inputs: HashMap<String, Val>,
  pub public_inputs_used: HashSet<String>,
  pub public_outputs: HashMap<String, Val>,
}

impl ValTrait for SummonIO {
  fn typeof_(&self) -> VsType {
    VsType::Object
  }

  fn to_number(&self) -> f64 {
    f64::NAN
  }

  fn to_index(&self) -> Option<usize> {
    None
  }

  fn is_primitive(&self) -> bool {
    false
  }

  fn is_truthy(&self) -> bool {
    true
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

  fn as_array_data(&self) -> Option<Rc<VsArray>> {
    None
  }

  fn as_class_data(&self) -> Option<Rc<VsClass>> {
    None
  }

  fn load_function(&self) -> LoadFunctionResult {
    LoadFunctionResult::NotAFunction
  }

  fn sub(&self, key: &Val) -> Result<Val, Val> {
    let Val::String(key) = key else {
      return Ok(Val::Undefined);
    };

    match key.as_ref() {
      "input" => Ok(INPUT.to_val()),
      "inputPublic" => Ok(INPUT_PUBLIC.to_val()),
      "output" => Ok(OUTPUT.to_val()),
      "outputPublic" => Ok(OUTPUT_PUBLIC.to_val()),
      _ => Ok(Val::Undefined),
    }
  }

  fn has(&self, key: &Val) -> Option<bool> {
    match self.sub(key) {
      Ok(Val::Undefined) => Some(false),
      Ok(_) => Some(true),
      Err(_) => None,
    }
  }

  fn submov(&mut self, _key: &Val, _value: Val) -> Result<(), Val> {
    Err("TODO: function subscript assignment".to_type_error())
  }

  fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[36m[Function]\x1b[39m")
  }

  fn codify(&self) -> String {
    "() => { [unavailable] }".to_string()
  }
}

impl std::fmt::Display for SummonIO {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[io]")
  }
}

static INPUT: NativeFunction = native_fn(|this, params| {
  let this_val = this.get();

  let Some(io) = val_dynamic_downcast::<SummonIO>(&this_val) else {
    return Err("Expected this to be Summon.IO".to_type_error());
  };

  let mut _io_data = io.data.borrow_mut();

  let (Some(from), Some(id), Some(type_)) = (params.first(), params.get(1), params.get(2)) else {
    return Err("Params (from, id, type) not provided".to_type_error());
  };

  let Val::String(_from) = from else {
    return Err("Expected `from` to be a string".to_type_error());
  };

  let Val::String(_id) = id else {
    return Err("Expected `id` to be a string".to_type_error());
  };

  let number_type = Val::make_object(&[
    ("about", "summon runtime type".to_val()),
    ("json", "number".to_val()),
  ]);

  let Ok(true) = op_triple_eq_impl(type_, &number_type) else {
    return Err(
      "Not implemented yet: type passed to io.input was something other than summon.number()"
        .to_internal_error(),
    );
  };

  Err("Not implemented yet: io.input".to_internal_error())
});

static INPUT_PUBLIC: NativeFunction = native_fn(|this, params| {
  let this_val = this.get();

  let Some(io) = val_dynamic_downcast::<SummonIO>(&this_val) else {
    return Err("Expected this to be Summon.IO".to_type_error());
  };

  let mut io_data = io.data.borrow_mut();

  let (Some(id), Some(type_)) = (params.first(), params.get(1)) else {
    return Err("Params (id, type) not provided".to_type_error());
  };

  let Val::String(id) = id else {
    return Err("Expected `id` to be a string".to_type_error());
  };

  let number_type = Val::make_object(&[
    ("about", "summon runtime type".to_val()),
    ("json", "number".to_val()),
  ]);

  let Ok(true) = op_triple_eq_impl(type_, &number_type) else {
    return Err(
      "Not implemented yet: type passed to io.publicInput was something other than summon.number()"
        .to_internal_error(),
    );
  };

  let Some(value) = io_data.public_inputs.get(&id.to_string()) else {
    return Err(format!("Missing public input: \"{}\"", id).to_error());
  };

  let value = value.clone();

  io_data.public_inputs_used.insert(id.to_string());

  Ok(value)
});

static OUTPUT: NativeFunction = native_fn(|this, params| {
  let this_val = this.get();

  let Some(io) = val_dynamic_downcast::<SummonIO>(&this_val) else {
    return Err("Expected this to be Summon.IO".to_type_error());
  };

  let mut _io_data = io.data.borrow_mut();

  let (Some(to), Some(id), Some(value)) = (params.first(), params.get(1), params.get(2)) else {
    return Err("Params (to, id, value) not provided".to_type_error());
  };

  let Val::String(_to) = to else {
    return Err(
      "Expected `to` to be a string (not implemented yet: array of strings)".to_type_error(),
    );
  };

  let Val::String(_id) = id else {
    return Err("Expected `id` to be a string".to_type_error());
  };

  if value.typeof_() != VsType::Number {
    return Err("Non-number outputs are not yet supported".to_type_error());
  }

  Err("Not implemented yet: io.output".to_internal_error())
});

static OUTPUT_PUBLIC: NativeFunction = native_fn(|this, params| {
  let this_val = this.get();

  let Some(io) = val_dynamic_downcast::<SummonIO>(&this_val) else {
    return Err("Expected this to be Summon.IO".to_type_error());
  };

  let mut io_data = io.data.borrow_mut();

  let (Some(id), Some(value)) = (params.first(), params.get(1)) else {
    return Err("Params (id, value) not provided".to_type_error());
  };

  let Val::String(id) = id else {
    return Err("Expected id to be a string".to_type_error());
  };

  if value.typeof_() != VsType::Number {
    return Err("Non-number outputs are not yet supported".to_type_error());
  }

  io_data.public_outputs.insert(id.to_string(), value.clone());

  Ok(Val::Undefined)
});
