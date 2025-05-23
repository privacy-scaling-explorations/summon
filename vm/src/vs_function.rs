use std::rc::Rc;

use crate::builtins::internal_error_builtin::ToInternalError;
use crate::bytecode::{Bytecode, DecoderMaker};
use crate::make_generator_frame::MakeGeneratorFrame;
use crate::vs_value::ToVal;

use super::bytecode_decoder::BytecodeDecoder;
use super::bytecode_stack_frame::BytecodeStackFrame;
use super::stack_frame::StackFrame;
use super::vs_value::Val;

#[derive(Debug, Clone)]
pub struct VsFunction {
  pub bytecode: Rc<Bytecode>,
  pub meta_pos: Option<usize>,
  pub is_generator: bool,
  pub register_count: usize,
  pub parameter_count: usize,
  pub start: usize,
  pub binds: Vec<Val>,
}

impl VsFunction {
  pub fn bind(&self, params: Vec<Val>) -> VsFunction {
    let mut new_binds = self.binds.clone();

    for p in params {
      new_binds.push(p);
    }

    VsFunction {
      bytecode: self.bytecode.clone(),
      meta_pos: self.meta_pos,
      is_generator: self.is_generator,
      register_count: self.register_count,
      parameter_count: self.parameter_count,
      start: self.start,
      binds: new_binds,
    }
  }

  pub fn content_hash(&self) -> Result<[u8; 32], Val> {
    match self.meta_pos {
      Some(p) => match self.bytecode.decoder(p).decode_meta().content_hash {
        Some(content_hash) => Ok(content_hash),
        None => Err("content_hash missing".to_internal_error()),
      },
      None => Err("Can't get content_hash without meta_pos".to_internal_error()),
    }
  }

  pub fn make_bytecode_frame(&self) -> BytecodeStackFrame {
    let mut registers: Vec<Val> = Vec::with_capacity(self.register_count - 1);

    registers.push(Val::Undefined);
    registers.push(Val::Undefined);

    for bind_val in &self.binds {
      registers.push(bind_val.clone());
    }

    while registers.len() < registers.capacity() {
      registers.push(Val::Void);
    }

    BytecodeStackFrame {
      decoder: BytecodeDecoder {
        bytecode: self.bytecode.clone(),
        pos: self.start,
      },
      registers,
      const_this: true,
      param_start: self.binds.len() + 2,
      param_end: self.parameter_count + 2,
      this_target: None,
      return_target: None,
      catch_setting: None,
      fork_info: None,
    }
  }

  pub fn make_frame(&self) -> StackFrame {
    let frame = self.make_bytecode_frame();

    match self.is_generator {
      false => Box::new(frame),
      true => Box::new(MakeGeneratorFrame::new(frame)),
    }
  }
}

impl ToVal for VsFunction {
  fn to_val(self) -> Val {
    Val::Function(Rc::new(self))
  }
}
