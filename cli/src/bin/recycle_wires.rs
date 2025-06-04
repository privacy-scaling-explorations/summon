// wire_recycler.rs — read an Extended‑Bristol circuit from stdin, recycle
// registers, and emit the new circuit.
//
// * `BristolCircuit` is now `Display`‑able; `fmt()` writes the full text
//   representation, so callers can simply `println!("{}", circuit)`.
// * The old inherent `to_string` has been removed.
//
// ─────────────────────────────────────────────────────────────────────────────
use std::{
  collections::{HashMap, VecDeque},
  error::Error,
  fmt::{self, Display, Formatter},
  fs::File,
  io::{self, Read, Write as IoWrite},
  path::Path,
};

/// One gate in Extended‑Bristol order  `k l <ins…> <outs…> OP`.
#[derive(Debug, Clone)]
pub struct Gate {
  pub k: usize,
  pub l: usize,
  pub ins: Vec<usize>,
  pub outs: Vec<usize>,
  pub op: String,
}

/// Header information plus raw header lines (except the first line, which will
/// be regenerated from the updated counts).
#[derive(Debug, Clone)]
pub struct Header {
  /// The original header lines *including* the first line.
  raw: Vec<String>,
  /// Parsed counts (kept in sync by `Display`).
  ngates: usize,
  nwires: usize,
}

/// A parsed Extended‑Bristol circuit.
#[derive(Debug, Clone)]
pub struct BristolCircuit {
  pub header: Header,
  pub gates: Vec<Gate>,
}

// ────────────────────────────── Display impl ───────────────────────────────
impl Display for BristolCircuit {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    // First header line (regenerated with new counts).
    writeln!(f, "{} {}", self.header.ngates, self.header.nwires)?;
    // Remaining header lines verbatim.
    for line in self.header.raw.iter().skip(1) {
      writeln!(f, "{}", line)?;
    }
    // Gate lines.
    for g in &self.gates {
      write!(f, "{} {}", g.k, g.l)?;
      for &w in &g.ins {
        write!(f, " {}", w)?;
      }
      for &w in &g.outs {
        write!(f, " {}", w)?;
      }
      writeln!(f, " {}", g.op)?;
    }
    Ok(())
  }
}

// ────────────────────────────── Parsing ────────────────────────────────────
fn split_sections(raw: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
  let mut lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
  if lines.is_empty() {
    return Err("empty file".into());
  }

  let mut first_parts = lines[0].split_whitespace();
  let ngates: usize = first_parts.next().ok_or("missing gate count")?.parse()?;
  let _nwires: usize = first_parts.next().ok_or("missing wire count")?.parse()?;

  let mut gate_lines = Vec::with_capacity(ngates);
  let mut nonblank = 0usize;
  for (idx, line) in lines.iter().enumerate().rev() {
    if line.trim().is_empty() {
      continue;
    }
    nonblank += 1;
    if nonblank <= ngates {
      gate_lines.push(line.clone());
    } else {
      lines.truncate(idx + 1);
      break;
    }
  }
  if gate_lines.len() != ngates {
    return Err(format!("expected {} gates, got {}", ngates, gate_lines.len()).into());
  }
  gate_lines.reverse();
  Ok((lines, gate_lines))
}

fn parse_gates(lines: &[String]) -> Result<Vec<Gate>, Box<dyn Error>> {
  let mut gates = Vec::with_capacity(lines.len());
  for ln in lines {
    if ln.trim().is_empty() {
      continue;
    }
    let mut parts = ln.split_whitespace();
    let k: usize = parts.next().ok_or("gate missing k")?.parse()?;
    let l: usize = parts.next().ok_or("gate missing l")?.parse()?;
    let mut ins = Vec::with_capacity(k);
    for _ in 0..k {
      ins.push(parts.next().ok_or("missing in‑wire")?.parse()?);
    }
    let mut outs = Vec::with_capacity(l);
    for _ in 0..l {
      outs.push(parts.next().ok_or("missing out‑wire")?.parse()?);
    }
    let op = parts.next().ok_or("missing op")?.to_string();
    gates.push(Gate {
      k,
      l,
      ins,
      outs,
      op,
    });
  }
  Ok(gates)
}

impl BristolCircuit {
  /// Parse from raw text.
  fn parse(raw: &str) -> Result<Self, Box<dyn Error>> {
    let (header_lines, gate_lines) = split_sections(raw)?;
    let gates = parse_gates(&gate_lines)?;
    let mut first = header_lines[0].split_whitespace();
    let ngates: usize = first.next().unwrap().parse()?;
    let nwires: usize = first.next().unwrap().parse()?;
    Ok(Self {
      header: Header {
        raw: header_lines,
        ngates,
        nwires,
      },
      gates,
    })
  }
}

/// All original input-wire indices from the header (0 … total_inputs-1).
fn input_len(hdr: &Header) -> Result<usize, Box<dyn Error>> {
  let lens = input_lengths(hdr)?; // helper from the TS backend
  Ok(lens.iter().sum())
}

/// Produce a *new* circuit whose wires are recycled.
fn recycle(circ: &BristolCircuit) -> Result<BristolCircuit, Box<dyn Error>> {
  let in_len = input_len(&circ.header)?;
  let out_len = output_length(&circ.header)?;

  let is_output_wire = |w: usize| w >= circ.header.nwires - out_len;

  let mut wire_map = HashMap::<usize, usize>::new();
  let mut next_wire: usize = 0;
  let mut recycling_pool = VecDeque::<usize>::new();

  #[allow(clippy::explicit_counter_loop)]
  for i in 0..in_len {
    wire_map.insert(i, next_wire);
    next_wire += 1;
  }

  // old wire id => last gate as input
  let mut last_uses_by_wire = HashMap::<usize, usize>::new();

  for (gate_index, gate) in circ.gates.iter().enumerate() {
    for in_ in &gate.ins {
      last_uses_by_wire.insert(*in_, gate_index);
    }
  }

  for i in 0..in_len {
    if !last_uses_by_wire.contains_key(&i) {
      let mapped = *wire_map.get(&i).expect("expected input wire to be mapped");
      assert_eq!(i, mapped);

      // if the input wire is not used, put the mapped wire in the recycling pool
      recycling_pool.push_back(mapped);
      eprintln!("input wire {} was not used", i);
    }
  }

  for i in 0..out_len {
    let old_wire_id = circ.header.nwires - out_len + i;

    // pretend the output wire is used again after the end of the circuit
    // (since it kinda is, just not by a gate)
    // this prevents us putting output wires into the recycling pool
    last_uses_by_wire.insert(old_wire_id, circ.gates.len());
  }

  let mut last_uses_by_gate = vec![vec![]; circ.gates.len() + 1];

  for (old_wire_id, gate_index) in last_uses_by_wire {
    last_uses_by_gate[gate_index].push(old_wire_id);
  }

  for (gate_index, gate) in circ.gates.iter().enumerate() {
    for out in &gate.outs {
      if is_output_wire(*out) {
        // don't map output wires here
        // we do that later when we know where to put them
        continue;
      }

      let out_mapped = match recycling_pool.pop_front() {
        Some(recycled) => recycled,
        None => {
          let new_wire_id = next_wire;
          next_wire += 1;
          new_wire_id
        }
      };

      wire_map.insert(*out, out_mapped);
    }

    for expired_old_wire_id in &last_uses_by_gate[gate_index] {
      let mapped = wire_map
        .get(expired_old_wire_id)
        .expect("expired wire should have been mapped");

      recycling_pool.push_back(*mapped);
    }
  }

  // Now that all other wires are mapped, we can map the output wires to
  // where we now know the new wires end
  for i in 0..out_len {
    let old_wire_id = circ.header.nwires - out_len + i;
    let new_wire_id = next_wire;
    next_wire += 1;
    wire_map.insert(old_wire_id, new_wire_id);
  }

  let mut new_gates = Vec::<Gate>::new();

  for gate in &circ.gates {
    new_gates.push(Gate {
      k: gate.k,
      l: gate.l,
      ins: gate
        .ins
        .iter()
        .map(|w| *wire_map.get(w).expect("unmapped wire"))
        .collect(),
      outs: gate
        .outs
        .iter()
        .map(|w| *wire_map.get(w).expect("unmapped wire"))
        .collect(),
      op: gate.op.clone(),
    });
  }

  let ngates = new_gates.len();
  let nwires = next_wire;
  let mut new_raw_lines = vec![format!("{ngates} {nwires}")];

  for i in 1..circ.header.raw.len() {
    new_raw_lines.push(circ.header.raw[i].clone());
  }

  Ok(BristolCircuit {
    header: Header {
      raw: new_raw_lines,
      ngates: new_gates.len(),
      nwires: next_wire,
    },
    gates: new_gates,
  })
}

/// Parse the second header line: “p n₀ n₁ …”.
fn input_lengths(hdr: &Header) -> Result<Vec<usize>, Box<dyn Error>> {
  let parts: Vec<_> = hdr
    .raw
    .get(1)
    .ok_or("missing input header")?
    .split_whitespace()
    .collect();
  let p: usize = parts.first().ok_or("bad input header")?.parse()?;
  if parts.len() != p + 1 {
    return Err("input counts mismatch".into());
  }
  Ok(
    parts[1..]
      .iter()
      .map(|s| s.parse::<usize>().unwrap())
      .collect(),
  )
}

/// Parse the third header line to learn total output length.
fn output_length(hdr: &Header) -> Result<usize, Box<dyn Error>> {
  let parts: Vec<_> = hdr
    .raw
    .get(2)
    .ok_or("missing output header")?
    .split_whitespace()
    .collect();
  let q: usize = parts.first().ok_or("bad output header")?.parse()?;
  if parts.len() != q + 1 {
    return Err("output counts mismatch".into());
  }
  Ok(parts[1..].iter().map(|s| s.parse::<usize>().unwrap()).sum())
}

/// Generate TypeScript function for circuit
///
/// The generated TS looks like:
///   const gates = [ [dst,'OP',in0,in1], … ];
///   export default function stem(input0:…, …) { … }
fn to_typescript(c: &BristolCircuit, stem: &str) -> Result<String, Box<dyn Error>> {
  use std::fmt::Write as _;

  let inputs = input_lengths(&c.header)?;
  let out_len = output_length(&c.header)?;
  let out_base = c.header.nwires - out_len; // contiguous after recycle()

  let mut ts = String::new();

  // ── gates array ───────────────────────────────────────────────────────
  ts.push_str("const gates = [\n");
  for g in &c.gates {
    match g.op.as_str() {
      "INV" if g.l == 1 => {
        writeln!(ts, "  [{}, 'INV', {}],", g.outs[0], g.ins[0])?;
      }
      "XOR" | "AND" if g.k == 2 && g.l == 1 => {
        writeln!(
          ts,
          "  [{}, '{}', {}, {}],",
          g.outs[0], g.op, g.ins[0], g.ins[1]
        )?;
      }
      op => return Err(format!("unsupported op {op}").into()),
    }
  }
  ts.push_str("];\n\n");

  // ── function prelude ──────────────────────────────────────────────────
  let params: Vec<_> = inputs
    .iter()
    .enumerate()
    .map(|(i, _)| format!("input{i}: boolean[]"))
    .collect();
  writeln!(
    ts,
    "export default function {stem}({}): boolean[] {{",
    params.join(", ")
  )?;

  // length checks
  for (i, &len) in inputs.iter().enumerate() {
    writeln!(
      ts,
      "  if (input{i}.length !== {len}) throw new Error(\"input{i} length\");"
    )?;
  }

  // copy inputs into `w`
  writeln!(ts, "  let w: boolean[] = [];")?;
  writeln!(ts, "  {{")?;
  writeln!(ts, "    let off = 0;")?;
  for (i, &len) in inputs.iter().enumerate() {
    writeln!(
      ts,
      "    for (let j = 0; j < {len}; ++j) w[off + j] = input{i}[j];"
    )?;
    writeln!(ts, "    off += {len};")?;
  }
  writeln!(ts, "  }}")?;

  // gate interpreter
  writeln!(ts, "  for (const [dst, op, in0, in1] of gates as any) {{")?;
  writeln!(ts, "    switch (op) {{")?;
  writeln!(ts, "      case 'INV': w[dst] = !w[in0]; break;")?;
  writeln!(ts, "      case 'XOR': w[dst] = w[in0] !== w[in1]; break;")?;
  writeln!(ts, "      case 'AND': w[dst] = w[in0] && w[in1]; break;")?;
  writeln!(ts, "    }}")?;
  writeln!(ts, "  }}")?;

  // outputs
  writeln!(ts, "  return w.slice({out_base}, {out_base} + {out_len});")?;
  writeln!(ts, "}}")?;

  Ok(ts)
}

// ───────────────────────────── CLI driver ──────────────────────────────────
fn main() -> Result<(), Box<dyn Error>> {
  // very small arg parser:  -i FILE  -o FILE
  let mut inp: Option<String> = None;
  let mut outp: Option<String> = None;
  let mut it = std::env::args().skip(1);
  while let Some(a) = it.next() {
    match a.as_str() {
      "-i" => inp = it.next(),
      "-o" => outp = it.next(),
      _ => return Err(format!("unknown arg {a}").into()),
    }
  }
  let infile = inp.ok_or("missing -i")?;
  let outfile = outp.ok_or("missing -o")?;

  // read
  let mut raw = String::new();
  if infile == "-" {
    io::stdin().read_to_string(&mut raw)?;
  } else {
    File::open(&infile)?.read_to_string(&mut raw)?;
  }
  // process
  let circ = BristolCircuit::parse(&raw)?;
  let recycled = recycle(&circ)?;

  // write
  if outfile == "-" {
    print!("{recycled}");
    return Ok(());
  }
  let path = Path::new(&outfile);
  let mut f = File::create(path)?;
  if path.extension().and_then(|s| s.to_str()) == Some("ts") {
    let src = to_typescript(&recycled, path.file_stem().unwrap().to_str().unwrap())?;
    f.write_all(src.as_bytes())?;
  } else {
    write!(f, "{recycled}")?;
  }
  Ok(())
}
