// wire_recycler.rs — read an Extended‑Bristol circuit from stdin, recycle
// registers, and emit the new circuit.
//
// * `BristolCircuit` is now `Display`‑able; `fmt()` writes the full text
//   representation, so callers can simply `println!("{}", circuit)`.
// * The old inherent `to_string` has been removed.
//
// ─────────────────────────────────────────────────────────────────────────────
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};

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
            for &w in &g.ins { write!(f, " {}", w)?; }
            for &w in &g.outs { write!(f, " {}", w)?; }
            writeln!(f, " {}", g.op)?;
        }
        Ok(())
    }
}

// ────────────────────────────── Parsing ────────────────────────────────────
fn split_sections(raw: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    let mut lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
    if lines.is_empty() { return Err("empty file".into()); }

    let mut first_parts = lines[0].split_whitespace();
    let ngates: usize = first_parts.next().ok_or("missing gate count")?.parse()?;
    let _nwires: usize = first_parts.next().ok_or("missing wire count")?.parse()?;

    let mut gate_lines = Vec::with_capacity(ngates);
    let mut nonblank = 0usize;
    for (idx, line) in lines.iter().enumerate().rev() {
        if line.trim().is_empty() { continue; }
        nonblank += 1;
        if nonblank <= ngates { gate_lines.push(line.clone()); }
        else { lines.truncate(idx + 1); break; }
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
        if ln.trim().is_empty() { continue; }
        let mut parts = ln.split_whitespace();
        let k: usize = parts.next().ok_or("gate missing k")?.parse()?;
        let l: usize = parts.next().ok_or("gate missing l")?.parse()?;
        let mut ins = Vec::with_capacity(k);
        for _ in 0..k { ins.push(parts.next().ok_or("missing in‑wire")?.parse()?); }
        let mut outs = Vec::with_capacity(l);
        for _ in 0..l { outs.push(parts.next().ok_or("missing out‑wire")?.parse()?); }
        let op = parts.next().ok_or("missing op")?.to_string();
        gates.push(Gate { k, l, ins, outs, op });
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
        Ok(Self { header: Header { raw: header_lines, ngates, nwires }, gates })
    }
}

// ────────────────────── Recycling Algorithm ───────────────────────────────
fn last_uses(gates: &[Gate]) -> HashMap<usize, usize> {
    let mut last = HashMap::new();
    for (idx, g) in gates.iter().enumerate() {
        for &w in &g.ins { last.insert(w, idx); }
    }
    last
}

/// Produce a *new* circuit whose wires are recycled.
fn recycle(circ: &BristolCircuit) -> BristolCircuit {
    // Identify input wires (never produced).
    let mut produced = HashSet::<usize>::new();
    for g in &circ.gates { produced.extend(&g.outs); }
    let mut inputs = HashSet::<usize>::new();
    for g in &circ.gates {
        for &w in &g.ins { if !produced.contains(&w) { inputs.insert(w); } }
    }

    let last = last_uses(&circ.gates);
    let mut map = HashMap::<usize, usize>::new();
    for &w in &inputs { map.insert(w, w); }

    let mut next_reg = inputs.iter().max().map(|&m| m + 1).unwrap_or(0);
    let mut free: VecDeque<usize> = VecDeque::new();
    let mut new_gates = Vec::with_capacity(circ.gates.len());

    for (idx, g) in circ.gates.iter().enumerate() {
        let mut ins_regs = Vec::with_capacity(g.k);
        for &orig in &g.ins { ins_regs.push(*map.get(&orig).expect("wire before def")); }
        let mut outs_regs = Vec::with_capacity(g.l);
        for &orig in &g.outs {
            let reg = free.pop_front().unwrap_or_else(|| { let r = next_reg; next_reg += 1; r });
            map.insert(orig, reg);
            outs_regs.push(reg);
        }
        for &orig in &g.ins {
            if last.get(&orig).copied() == Some(idx) && !inputs.contains(&orig) {
                if let Some(reg) = map.remove(&orig) { free.push_back(reg); }
            }
        }
        new_gates.push(Gate { k: g.k, l: g.l, ins: ins_regs, outs: outs_regs, op: g.op.clone() });
    }

    // Live registers that are not preserved inputs become outputs.
    let mut outs: Vec<usize> = map.iter().filter(|(o, _)| !inputs.contains(*o)).map(|(_, &r)| r).collect();
    outs.sort_unstable();
    let contiguous = outs.windows(2).all(|w| w[1] == w[0] + 1) && outs.first().copied() == Some(next_reg - outs.len());

    if !contiguous {
        let start = next_reg;
        let mut rename = HashMap::<usize, usize>::new();
        for (i, &r) in outs.iter().enumerate() { rename.insert(r, start + i); }
        next_reg += outs.len();
        for gate in &mut new_gates {
            for r in &mut gate.outs { if let Some(&nr) = rename.get(r) { *r = nr; } }
        }
    }

    let mut new_header = circ.header.clone();
    new_header.ngates = new_gates.len();
    new_header.nwires = next_reg;

    BristolCircuit { header: new_header, gates: new_gates }
}

// ──────────────────────────── Driver ───────────────────────────────────────
fn main() -> Result<(), Box<dyn Error>> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    let circ = BristolCircuit::parse(&buf)?;
    let recycled = recycle(&circ);
    print!("{}", recycled);
    Ok(())
}
