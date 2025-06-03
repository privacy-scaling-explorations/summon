// wire_recycler.rs — read an Extended‑Bristol circuit from stdin and emit a
// register‑recycled circuit **without** inserting BUF copy gates.
//
// Input format (Extended‑Bristol):
//   <k> <l> <in‑wires…> <out‑wires…> <OP>
// Header may be any length; only the first line ("#gates #wires") is updated.
//
// Strategy
// ────────
// 1.  Scan gates once to find each wire’s last *use* as an input.
// 2.  While streaming through the gates, reuse registers that are no longer
//     live.  Input wires keep their original labels.
// 3.  When we finish, whatever wires are still defined but never consumed are
//     the *functional* outputs.  We *rename* those registers to a fresh,
//     contiguous block at the end of the numbering space, patching their
//     defining gates in‑place.  No BUF gates are needed.
//
// Build & run:
//     cargo run --release < in.bristol > out.bristol
// ─────────────────────────────────────────────────────────────────────────────
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::io::{self, Read};

#[derive(Debug, Clone)]
struct Gate {
    k: usize,
    l: usize,
    ins: Vec<usize>,
    outs: Vec<usize>,
    op: String,
}

/// Split the raw file into (header lines, gate lines).
fn split_sections(raw: &str) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    let mut lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
    if lines.is_empty() {
        return Err("empty file".into());
    }

    // First line = "#gates #wires".
    let mut first_parts = lines[0].split_whitespace();
    let ngates: usize = first_parts
        .next()
        .ok_or("first line missing gate count")?
        .parse()?;
    let _ = first_parts
        .next()
        .ok_or("first line missing wire count")?
        .parse::<usize>()?;

    // Count back `ngates` non‑blank lines to obtain gate section.
    let mut gate_lines = Vec::with_capacity(ngates);
    let mut nonblank = 0;
    for (idx, line) in lines.iter().enumerate().rev() {
        if line.trim().is_empty() {
            continue;
        }
        nonblank += 1;
        if nonblank <= ngates {
            gate_lines.push(line.clone());
        } else {
            lines.truncate(idx + 1); // header ends here (inclusive)
            break;
        }
    }
    if gate_lines.len() != ngates {
        return Err(format!("expected {} gates, found {}", ngates, gate_lines.len()).into());
    }
    gate_lines.reverse();
    Ok((lines, gate_lines))
}

/// Parse gate lines.
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
            ins.push(parts.next().ok_or("gate missing input wire")?.parse()?);
        }
        let mut outs = Vec::with_capacity(l);
        for _ in 0..l {
            outs.push(parts.next().ok_or("gate missing output wire")?.parse()?);
        }
        let op = parts.next().ok_or("gate missing op")?.to_string();
        gates.push(Gate { k, l, ins, outs, op });
    }
    Ok(gates)
}

/// Last index where each *wire* appears as an input.
fn last_uses(gates: &[Gate]) -> HashMap<usize, usize> {
    let mut last = HashMap::new();
    for (idx, g) in gates.iter().enumerate() {
        for &w in &g.ins {
            last.insert(w, idx);
        }
    }
    last
}

fn recycle(header: Vec<String>, gates: Vec<Gate>) -> Result<String, Box<dyn Error>> {
    // Identify input wires (never produced by any gate).
    let mut produced = HashSet::<usize>::new();
    for g in &gates {
        produced.extend(&g.outs);
    }
    let mut inputs = HashSet::<usize>::new();
    for g in &gates {
        for &w in &g.ins {
            if !produced.contains(&w) {
                inputs.insert(w);
            }
        }
    }

    let last = last_uses(&gates);

    // Map original wire → current register.
    let mut map = HashMap::<usize, usize>::new();
    for &w in &inputs {
        map.insert(w, w); // preserve labels for inputs
    }

    let mut next_reg = inputs.iter().max().map(|&m| m + 1).unwrap_or(0);
    let mut freelist: VecDeque<usize> = VecDeque::new();

    let mut new_gates: Vec<Gate> = Vec::with_capacity(gates.len());

    for (idx, g) in gates.iter().enumerate() {
        // Translate inputs.
        let mut ins_regs = Vec::with_capacity(g.k);
        for &orig in &g.ins {
            ins_regs.push(*map.get(&orig).ok_or("wire used before assignment")?);
        }

        // Allocate outputs.
        let mut outs_regs = Vec::with_capacity(g.l);
        for &orig in &g.outs {
            let reg = if let Some(r) = freelist.pop_front() { r } else { let r = next_reg; next_reg += 1; r };
            map.insert(orig, reg);
            outs_regs.push(reg);
        }

        // Free inputs whose last use is here and are not preserved inputs.
        for &orig in &g.ins {
            if last.get(&orig).copied() == Some(idx) && !inputs.contains(&orig) {
                if let Some(reg) = map.remove(&orig) {
                    freelist.push_back(reg);
                }
            }
        }

        new_gates.push(Gate { k: g.k, l: g.l, ins: ins_regs, outs: outs_regs, op: g.op.clone() });
    }

    // Determine functional outputs (excluding preserved inputs).
    let mut outputs: Vec<usize> = map
        .iter()
        .filter(|(orig, _)| !inputs.contains(*orig))
        .map(|(_, &reg)| reg)
        .collect();
    outputs.sort_unstable();

    // If already contiguous and at the end, we’re done.
    let contiguous = outputs
        .windows(2)
        .all(|w| w[1] == w[0] + 1) && outputs.first().copied() == Some(next_reg - outputs.len());

    if !contiguous {
        // Reserve fresh contiguous block.
        let start = next_reg;
        let mut rename = HashMap::<usize, usize>::new();
        for (i, &reg) in outputs.iter().enumerate() {
            rename.insert(reg, start + i);
        }
        next_reg += outputs.len();

        // Patch defining gates’ *outs* (outputs never appear as inputs again).
        for gate in &mut new_gates {
            for r in &mut gate.outs {
                if let Some(&new_r) = rename.get(r) {
                    *r = new_r;
                }
            }
        }
        outputs = (start..start + rename.len()).collect();
    }

    // ─── Emit ────────────────────────────────────────────────────────────────
    let new_gate_count = new_gates.len();
    let new_wire_count = next_reg;

    let mut out = String::new();
    out.push_str(&format!("{} {}\n", new_gate_count, new_wire_count));
    for line in header.iter().skip(1) {
        out.push_str(line);
        out.push('\n');
    }

    for g in &new_gates {
        out.push_str(&format!("{} {}", g.k, g.l));
        for &w in &g.ins { out.push(' '); out.push_str(&w.to_string()); }
        for &w in &g.outs { out.push(' '); out.push_str(&w.to_string()); }
        out.push(' '); out.push_str(&g.op); out.push('\n');
    }

    Ok(out)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    let (header, gate_lines) = split_sections(&buf)?;
    let gates = parse_gates(&gate_lines)?;
    let recycled = recycle(header, gates)?;
    print!("{}", recycled);
    Ok(())
}
