#[cfg(test)]
mod tests_ {
  use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
  };

  use summon_vm::vs_value::{ToVal, Val};

  use crate::{compile, resolve_entry_path::resolve_entry_path, DiagnosticsByPath};

  #[test]
  fn test_annotations() {
    let test_cases = find_test_cases("../examples");

    for test_case in &test_cases {
      let TestCase {
        path,
        descriptor,
        public_inputs,
        input,
        expected_output,
      } = test_case;

      println!("Test {}: {}", path, descriptor);

      let path = resolve_entry_path(path);

      let compile_result = compile(public_inputs, path.clone(), |p| {
        fs::read_to_string(p).map_err(|e| e.to_string())
      });

      let diagnostics = match &compile_result {
        Ok(compile_ok) => &compile_ok.diagnostics,
        Err(compile_err) => &compile_err.diagnostics,
      };

      println!("{}", DiagnosticsByPath(diagnostics.clone()));

      let circuit = compile_result.expect("Compile failed").circuit;

      let inputs = circuit
        .inputs
        .iter()
        .map(|(name, i)| (name.clone(), input[*i]))
        .collect::<BTreeMap<_, _>>();

      let outputs = circuit.eval(&inputs);

      let mut output_names = circuit.outputs.iter().collect::<Vec<_>>();
      output_names.sort_by(|(_, id_a), (_, id_b)| id_a.cmp(id_b));

      let ordered_outputs = output_names
        .iter()
        .map(|(name, _)| outputs.get(*name).unwrap())
        .collect::<Vec<_>>();

      let output_name_to_index = output_names
        .iter()
        .enumerate()
        .map(|(i, (name, _))| ((*name).clone(), i))
        .collect::<BTreeMap<_, _>>();

      for (name, value) in &outputs {
        let wire_id = output_name_to_index[name];

        assert_eq!(
          *value,
          expected_output[wire_id],
          "Test: {}: {}: Output mismatch for {}: expected {}, got {} ({:?} vs {:?})",
          path.path,
          descriptor,
          name,
          expected_output[wire_id],
          value,
          expected_output,
          ordered_outputs
        );
      }
    }
  }

  #[derive(Debug)]
  struct TestCase {
    path: String,
    descriptor: String,
    public_inputs: HashMap<String, Val>,
    input: Vec<usize>,
    expected_output: Vec<usize>,
  }

  fn parse_test_case(path: &str, line: &str) -> Option<TestCase> {
    let line = line.trim();

    if !line.starts_with("//! test ") {
      return None;
    }

    // strip prefix
    let mut rest = line["//! test ".len()..].trim_start();
    let descriptor = rest.to_string();

    // parse optional public_inputs
    let mut public_inputs = HashMap::new();
    if rest.starts_with('{') {
      // find matching '}'
      let end = rest
        .find('}')
        .expect("missing closing `}` for public_inputs block");
      let block = &rest[1..end]; // inside { ... }

      for pair in block.split(',') {
        let mut kv = pair.splitn(2, ':');
        let key = kv
          .next()
          .expect("empty key in public_inputs")
          .trim()
          .to_string();
        let val_str = kv
          .next()
          .unwrap_or_else(|| panic!("missing `:` in public_inputs pair `{}`", pair))
          .trim();
        let val = val_str
          .parse::<usize>()
          .unwrap_or_else(|_| panic!("invalid usize `{}` in public_inputs", val_str));
        public_inputs.insert(key, (val as f64).to_val());
      }

      // advance rest past the '}' and any following whitespace
      rest = rest[end + 1..].trim_start();
    }

    // now rest should be "[... ] => [... ]"
    let parts: Vec<&str> = rest.split("=>").collect();
    assert!(
      parts.len() == 2,
      "expected one `=>` separating input and output, got `{}`",
      rest
    );

    // parse input vector
    let input = parts[0]
      .trim()
      .trim_start_matches('[')
      .trim_end_matches(']')
      .split(',')
      .map(|s| {
        let t = s.trim();
        t.parse::<usize>()
          .unwrap_or_else(|_| panic!("invalid usize `{}` in input array", t))
      })
      .collect();

    // parse expected_output vector
    let expected_output = parts[1]
      .trim()
      .trim_start_matches('[')
      .trim_end_matches(']')
      .split(',')
      .map(|s| {
        let t = s.trim();
        t.parse::<usize>()
          .unwrap_or_else(|_| panic!("invalid usize `{}` in expected_output array", t))
      })
      .collect();

    Some(TestCase {
      path: path.to_string(),
      descriptor,
      public_inputs,
      input,
      expected_output,
    })
  }

  fn find_test_cases(dir: &str) -> Vec<TestCase> {
    let mut test_cases = Vec::new();

    for path in read_dir_recursive(dir) {
      let content = fs::read_to_string(&path).expect("Unable to read file");

      for line in content.lines() {
        if let Some(test_case) = parse_test_case(path.to_str().unwrap(), line) {
          test_cases.push(test_case);
        }
      }
    }

    test_cases
  }

  fn read_dir_recursive(dir: &str) -> Vec<PathBuf> {
    let mut res = Vec::<PathBuf>::new();

    for entry in fs::read_dir(dir).expect("Directory not found") {
      let path = entry.expect("Unable to read entry").path();

      if path.is_file() {
        res.push(path);
      } else if path.is_dir() {
        res.append(&mut read_dir_recursive(path.to_str().unwrap()));
      }
    }

    res
  }
}
