use strum::EnumCount;
use strum_macros::{EnumCount, EnumString};

#[derive(EnumString, EnumCount, Clone, Copy)]
pub enum BuiltinName {
  Debug,
  Math,
  String,
  Number,
  Boolean,
  Array,

  #[allow(non_camel_case_types)]
  isFinite,

  #[allow(non_camel_case_types)]
  isNaN,

  #[allow(non_camel_case_types)]
  parseFloat,

  #[allow(non_camel_case_types)]
  parseInt,

  Error,
  TypeError,
  RangeError,
  InternalError,

  Symbol,
  SymbolIterator,

  BigInt,

  #[allow(non_camel_case_types)]
  console,

  #[allow(non_camel_case_types)]
  summon,
}

pub const BUILTIN_NAMES: [&str; BuiltinName::COUNT] = [
  "Debug",
  "Math",
  "String",
  "Number",
  "Boolean",
  "Array",
  "isFinite",
  "isNaN",
  "parseFloat",
  "parseInt",
  "Error",
  "TypeError",
  "RangeError",
  "InternalError",
  "Symbol",
  "SymbolIterator",
  "BigInt",
  "console",
  "summon",
];

pub const BUILTIN_COUNT: usize = BuiltinName::COUNT;

impl BuiltinName {
  pub fn to_code(&self) -> usize {
    *self as usize
  }
}
