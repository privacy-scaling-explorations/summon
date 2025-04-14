use std::{collections::HashMap, path::PathBuf};

use crate::{Diagnostic, DiagnosticLevel, ResolvedPath};

pub struct DiagnosticsByPath(pub HashMap<ResolvedPath, Vec<Diagnostic>>);

impl DiagnosticsByPath {
  pub fn has_errors(&self) -> bool {
    for diagnostic in self.0.values().flatten() {
      match diagnostic.level {
        DiagnosticLevel::Lint => {}
        DiagnosticLevel::Error | DiagnosticLevel::InternalError => return true,
        DiagnosticLevel::CompilerDebug => {}
      }
    }

    false
  }

  pub fn has_internal_errors(&self) -> bool {
    for diagnostic in self.0.values().flatten() {
      if diagnostic.level == DiagnosticLevel::InternalError {
        return true;
      }
    }

    false
  }
}

impl std::fmt::Display for DiagnosticsByPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for (file_path, file_diagnostics) in &self.0 {
      handle_file_diagnostics(f, &file_path.path, file_diagnostics)?;
    }

    Ok(())
  }
}

fn handle_file_diagnostics(
  f: &mut std::fmt::Formatter<'_>,
  file_path: &String,
  diagnostics: &Vec<Diagnostic>,
) -> std::fmt::Result {
  let path = 'b: {
    if file_path == "(str)" {
      // TODO: Fix this hack
      break 'b None;
    }

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let abs_path = PathBuf::from(file_path);

    Some(match abs_path.strip_prefix(&current_dir) {
      Ok(p) => p.into(),
      Err(_) => abs_path,
    })
  };

  let path_str = match path {
    Some(path) => path.to_string_lossy().to_string(),
    None => file_path.clone(),
  };

  let mut level_counts = HashMap::<DiagnosticLevel, usize>::new();

  let text = if file_path == "(str)" {
    None
  } else {
    Some(std::fs::read_to_string(file_path).unwrap())
  };

  for diagnostic in diagnostics {
    let (line, col) = match &text {
      Some(text) => pos_to_line_col(text, diagnostic.span.lo.0),
      None => (0, 0),
    };

    writeln!(
      f,
      "{}:{}:{}: {}: {}",
      path_str, line, col, diagnostic.level, diagnostic.message
    )?;

    let count = level_counts.entry(diagnostic.level).or_insert(0);
    *count += 1;
  }

  let error_count = level_counts.get(&DiagnosticLevel::Error).unwrap_or(&0);

  let internal_error_count = level_counts
    .get(&DiagnosticLevel::InternalError)
    .unwrap_or(&0);

  let total_error_count = error_count + internal_error_count;

  if total_error_count > 0 {
    writeln!(f, "\nFailed with {} error(s)", total_error_count)?;
  }

  Ok(())
}

fn pos_to_line_col(text: &str, pos: u32) -> (u32, u32) {
  let mut line = 1u32;
  let mut col = 1u32;

  for (i, c) in text.chars().enumerate() {
    if i as u32 == pos {
      break;
    }

    if c == '\n' {
      line += 1;
      col = 1;
    } else {
      col += 1;
    }
  }

  (line, col)
}
