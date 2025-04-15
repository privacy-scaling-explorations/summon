use std::{cell::RefCell, collections::BTreeMap, collections::HashMap, rc::Rc};

use crate::asm::Module;
use crate::summon_io::SummonIO;
use summon_common::InputDescriptor;
use summon_vm::circuit::MpcSettings;
use summon_vm::vs_value::{ToDynamicVal, Val};
use summon_vm::{
  circuit::Circuit, circuit_builder::CircuitBuilder, circuit_vm::CircuitVM,
  id_generator::IdGenerator, Bytecode, DecoderMaker,
};
use swc_common::DUMMY_SP;

use crate::{
  asm, assembler::assemble, diagnostic::DiagnosticLevel, gather_modules, link_module, Diagnostic,
  ResolvedPath,
};

pub struct CompileOk {
  pub circuit: Circuit,
  pub diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

#[derive(Debug)]
pub struct CompileErr {
  pub circuit: Option<Circuit>,
  pub diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

pub type CompileResult = Result<CompileOk, CompileErr>;

pub struct CompileLinkedModuleResult {
  pub module: Option<Module>,
  pub diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

pub fn compile<ReadFile>(
  public_inputs: &HashMap<String, Val>,
  path: ResolvedPath,
  read_file: ReadFile,
) -> CompileResult
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let CompileArtifacts {
    main_asm,
    main,
    mut diagnostics,
  } = get_compile_artifacts(path.clone(), read_file)?;

  if main_asm.parameters.len() != 1 {
    diagnostics.entry(path).or_default().push(Diagnostic {
      level: DiagnosticLevel::Error,
      message: format!(
        "number of main function arguments ({}) is not 1",
        main_asm.parameters.len()
      ),
      span: DUMMY_SP,
    });

    return Err(CompileErr {
      circuit: None,
      diagnostics,
    });
  }

  let id_gen = Rc::new(RefCell::new(IdGenerator::new()));
  let io = SummonIO::new(public_inputs, &id_gen);
  run(main, &io);

  for unused_input in io.unused_public_inputs() {
    let unused_path = ResolvedPath {
      path: "(public inputs)".to_string(),
    };

    diagnostics
      .entry(unused_path)
      .or_default()
      .push(Diagnostic {
        level: DiagnosticLevel::Lint,
        message: format!("Unused public input: {}", unused_input),
        span: DUMMY_SP,
      });
  }

  let (input_descriptors, outputs, builder) = build(io);
  let circuit = generate_circuit(input_descriptors, outputs, builder);

  if diagnostics.iter().any(|(_, path_diagnostics)| {
    path_diagnostics.iter().any(|diagnostic| {
      matches!(
        diagnostic.level,
        DiagnosticLevel::Error | DiagnosticLevel::InternalError
      )
    })
  }) {
    return Err(CompileErr {
      circuit: Some(circuit),
      diagnostics,
    });
  }

  Ok(CompileOk {
    circuit,
    diagnostics,
  })
}

pub fn compile_linked_module<ReadFile>(
  entry_point: ResolvedPath,
  read_file: ReadFile,
) -> CompileLinkedModuleResult
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let gm = gather_modules(entry_point.clone(), read_file);
  let mut link_module_result = link_module(&gm.entry_point, &gm.modules);

  let mut result = CompileLinkedModuleResult {
    module: link_module_result.module,
    diagnostics: gm.diagnostics,
  };

  result
    .diagnostics
    .entry(entry_point)
    .or_default()
    .append(&mut link_module_result.diagnostics);

  result
}

struct CompileArtifacts {
  main_asm: asm::Function,
  main: Val,
  diagnostics: HashMap<ResolvedPath, Vec<Diagnostic>>,
}

fn get_compile_artifacts<ReadFile>(
  path: ResolvedPath,
  read_file: ReadFile,
) -> Result<CompileArtifacts, CompileErr>
where
  ReadFile: Fn(&str) -> Result<String, String>,
{
  let gm = gather_modules(path.clone(), read_file);
  let mut link_module_result = link_module(&gm.entry_point, &gm.modules);

  let module = link_module_result.module;
  let mut diagnostics = gm.diagnostics;

  diagnostics
    .entry(path)
    .or_default()
    .append(&mut link_module_result.diagnostics);

  let module = match module {
    Some(module) => module,
    None => {
      return Err(CompileErr {
        circuit: None,
        diagnostics,
      })
    }
  };

  let asm_fn = get_asm_main(&module);

  let bytecode = Rc::new(Bytecode::new(assemble(&module)));

  let val = bytecode.decoder(0).decode_val(&mut vec![]);

  Ok(CompileArtifacts {
    main_asm: asm_fn.clone(),
    main: val,
    diagnostics,
  })
}

fn get_asm_main(module: &asm::Module) -> &asm::Function {
  let main_ptr = match &module.export_default {
    asm::Value::Pointer(ptr) => ptr,
    _ => panic!("Expected pointer"),
  };

  let fn_ = match resolve_ptr(module, main_ptr).unwrap() {
    asm::DefinitionContent::Function(fn_) => fn_,
    _ => panic!("Expected function"),
  };

  fn_
}

fn resolve_ptr<'a>(
  module: &'a asm::Module,
  ptr: &asm::Pointer,
) -> Option<&'a asm::DefinitionContent> {
  for defn in &module.definitions {
    if &defn.pointer == ptr {
      return Some(&defn.content);
    }
  }

  None
}

fn run(main: Val, io: &SummonIO) {
  let mut vm = CircuitVM::default();

  let res = vm.run(
    None,
    &mut Val::Undefined,
    main,
    vec![io.clone().to_dynamic_val()],
  );

  match &res {
    Ok(Val::Void | Val::Undefined) => {}
    Ok(return_value) => {
      println!("Program output: {}", return_value.pretty());
    }
    Err(err) => {
      eprintln!("Uncaught exception: {}", err.pretty());
      std::process::exit(1);
    }
  };
}

fn build(
  io: SummonIO,
) -> (
  Vec<InputDescriptor>,
  BTreeMap<String, usize>,
  CircuitBuilder,
) {
  let mut builder = CircuitBuilder::default();
  builder.include_inputs(&io.input_ids());

  let io_data = io.data.borrow();
  let input_descriptors = io_data.inputs.clone();
  let outputs = builder.include_outputs(&io_data.public_outputs);

  drop(io_data);
  drop(io);
  builder.drop_signal_data();

  (input_descriptors, outputs, builder)
}

fn generate_circuit(
  input_descriptors: Vec<InputDescriptor>,
  outputs: BTreeMap<String, usize>,
  builder: CircuitBuilder,
) -> Circuit {
  let mut inputs = BTreeMap::<String, usize>::new();
  for (i, desc) in input_descriptors.iter().enumerate() {
    inputs.insert(desc.name.clone(), i);
  }

  let mut constants = BTreeMap::<usize, usize>::new();
  for (value, wire_id) in &builder.constants {
    constants.insert(*wire_id, *value);
  }

  let outputs_vec = outputs.keys().cloned().collect::<Vec<_>>();

  Circuit {
    size: builder.wire_count,
    inputs,
    constants,
    outputs,
    mpc_settings: MpcSettings::from_io(&input_descriptors, outputs_vec),
    gates: builder.gates,
  }
}
