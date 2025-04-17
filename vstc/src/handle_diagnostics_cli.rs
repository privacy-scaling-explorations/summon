use std::collections::HashMap;

use serde_qs as qs;
use url::Url;

use summon_compiler::{Diagnostic, DiagnosticsByPath, ResolvedPath};

pub fn handle_diagnostics_cli(diagnostics: &HashMap<ResolvedPath, Vec<Diagnostic>>) {
  let dbp = DiagnosticsByPath(diagnostics.clone());

  print!("{}", dbp);

  if dbp.has_internal_errors() {
    println!();
    println!("===============================");
    println!("=== INTERNAL ERROR(S) FOUND ===");
    println!("===============================");
    println!();

    // Create a github issue link
    let mut url = Url::parse("https://github.com/voltrevo/summon/issues/new").unwrap();

    #[derive(serde::Serialize)]
    struct TitleAndBody {
      title: String,
      body: String,
    }

    let query_string = qs::to_string(&TitleAndBody {
      title: "Internal error(s) found".to_string(),
      body: format!(
        "Input:\n```\n(Please provide if you can)\n```\n\nOutput:\n```\n{}\n```",
        dbp
      ),
    })
    .unwrap();

    url.set_query(Some(&query_string));

    println!("This is a bug in summon, please consider reporting it:");
    println!();
    println!("{}", url);
    println!();
  }

  if dbp.has_errors() {
    std::process::exit(1);
  }
}
