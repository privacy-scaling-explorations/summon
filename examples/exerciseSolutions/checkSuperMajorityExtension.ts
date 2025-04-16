// Check for a Supermajority.
//
// Write a circuit which checks whether 2/3 or more ballots approved the motion (supermajority).
// Each ballot is a number, with 0 meaning 'no' and any other number meaning 'yes'.
// The circuit should return 1 to indicate the motion passes and 0 to indicate it fails.
//
// For example:
//! test { N: 10 } [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] => [0]
//! test { N: 10 } [1, 1, 1, 1, 1, 1, 1, 1, 1, 1] => [1]
//! test { N: 10 } [0, 7, 8, 0, 1, 1, 0, 0, 2, 2] => [0]
//! test { N: 10 } [0, 7, 8, 0, 1, 1, 0, 3, 2, 2] => [1]
//! test { N: 3 } [0, 0, 0] => [0]
//! test { N: 3 } [1, 1, 1] => [1]
//! test { N: 3 } [0, 1, 1] => [1]
//! test { N: 3 } [0, 0, 1] => [0]
//! test { N: 5 } [0, 0, 0, 0, 0] => [0]
//! test { N: 5 } [1, 1, 1, 1, 1] => [1]
//! test { N: 5 } [0, 1, 1, 1, 1] => [1]
//! test { N: 5 } [0, 0, 1, 1, 1] => [0]
//
// The format above is also used to check circuits with `cargo test`. Simply move them to their own
// line, similar to test annotations in `loopAdd.ts` and `greaterThan10.ts`.

import range from "../lib/range.ts";
import treeSum from "../lib/treeSum.ts";

export default function main(io: Summon.IO) {
  const N = io.inputPublic('N', summon.number());

  const ballots = range(0, N).map(
    i => io.input(`party${i}`, `ballot${i}`, summon.number())
  );

  io.outputPublic('result', impl(ballots) ? 1 : 0);
}

function impl(ballots: number[]) {
  const sum = treeSum(ballots, b => b !== 0 ? 1 : 0);
  return sum * 3 >= ballots.length * 2;
}
