use std::{
  collections::HashMap,
  fs::{self, File},
  io::BufWriter,
  path::Path,
};

use boolify::boolify;
use serde_json::to_string_pretty;
use summon_cli::handle_diagnostics_cli;
use summon_compiler::{bristol_depth, compile, resolve_entry_path, CompileOk};
use summon_vm::vs_value::Val;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if args.len() < 2 {
    eprintln!("Usage: summonc main.ts [--public-inputs json|FILE.json] [--boolify-width WIDTH]");
    std::process::exit(1);
  }

  let mut public_inputs_path = None;
  let mut boolify_width = None;

  for i in 2..args.len() {
    if args[i] == "--public-inputs" {
      public_inputs_path = Some(args.get(i + 1).expect("missing arg").clone());
    } else if args[i] == "--boolify-width" {
      boolify_width = Some(
        args
          .get(i + 1)
          .expect("missing arg")
          .parse::<usize>()
          .expect("invalid usize"),
      );
    }
  }

  let entry_point = resolve_entry_path(&args[1]);

  let public_inputs: HashMap<String, Val> = if let Some(path) = public_inputs_path {
    if path.get(0..1) == Some("{") {
      // if the first character is '{', we assume it's a json string
      let numbers = serde_json::from_str::<HashMap<String, usize>>(&path)
        .expect("Failed to parse public inputs string");

      numbers
        .into_iter()
        .map(|(k, v)| (k, Val::Number(v as f64)))
        .collect::<HashMap<_, _>>()
    } else {
      let path = Path::new(&path);

      if !path.exists() {
        eprintln!("Public inputs file does not exist: {}", path.display());
        std::process::exit(1);
      }

      let file = File::open(path).expect("Failed to open public inputs file");

      // only numbers for now
      let numbers = serde_json::from_reader::<_, HashMap<String, usize>>(file)
        .expect("Failed to parse public inputs file");

      numbers
        .into_iter()
        .map(|(k, v)| (k, Val::Number(v as f64)))
        .collect::<HashMap<_, _>>()
    }
  } else {
    HashMap::new()
  };

  let compile_result = compile(&public_inputs, entry_point, |path| {
    fs::read_to_string(path).map_err(|e| e.to_string())
  });

  let diagnostics = match &compile_result {
    Ok(ok) => &ok.diagnostics,
    Err(err) => &err.diagnostics,
  };

  handle_diagnostics_cli(diagnostics);

  let CompileOk {
    circuit,
    diagnostics: _,
  } = compile_result.expect("Error should have caused earlier exit");

  let output_dir = Path::new("output");

  if output_dir.exists() {
    fs::remove_dir_all(output_dir).unwrap();
  }

  fs::create_dir(output_dir).unwrap();

  let mut bristol_circuit = circuit.to_bristol();

  if let Some(boolify_width) = boolify_width {
    bristol_circuit = boolify(&bristol_circuit, boolify_width)
  }

  println!(
    "Wires: {}, Gates: {}, Depth: {}",
    bristol_circuit.wire_count,
    bristol_circuit.gates.len(),
    bristol_depth(&bristol_circuit),
  );

  bristol_circuit
    .write_bristol(&mut BufWriter::new(
      File::create("output/circuit.txt").unwrap(),
    ))
    .unwrap();
  println!("output/circuit.txt");

  fs::write(
    "output/circuit_info.json",
    to_string_pretty(&bristol_circuit.info).unwrap(),
  )
  .unwrap();
  println!("output/circuit_info.json");

  fs::write(
    "output/mpc_settings.json",
    to_string_pretty(&circuit.mpc_settings).unwrap(),
  )
  .unwrap();
  println!("output/mpc_settings.json");
}
