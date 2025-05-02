// Check for a Supermajority.
//
// Write a circuit which checks whether 2/3 or more ballots approved the motion (supermajority).
// Each ballot is a number, with 0 meaning 'no' and any other number meaning 'yes'.
// The circuit should return 1 to indicate the motion passes and 0 to indicate it fails.
//
// For example:
//! test { N: 10 } [false, false, false, false, false, false, false, false, false, false] => [false]
//! test { N: 10 } [ true,  true,  true,  true,  true,  true,  true,  true,  true,  true] => [ true]
//! test { N: 10 } [false,  true,  true, false,  true,  true, false, false,  true,  true] => [false]
//! test { N: 10 } [false,  true,  true, false,  true,  true, false,  true,  true,  true] => [ true]
//! test { N: 3 } [false, false, false] => [false]
//! test { N: 3 } [ true,  true,  true] => [ true]
//! test { N: 3 } [false,  true,  true] => [ true]
//! test { N: 3 } [false, false,  true] => [false]
//! test { N: 5 } [false, false, false, false, false] => [false]
//! test { N: 5 } [ true,  true,  true,  true,  true] => [ true]
//! test { N: 5 } [false,  true,  true,  true,  true] => [ true]
//! test { N: 5 } [false, false,  true,  true,  true] => [false]
//
// The format above is also used to check circuits with `cargo test`. Simply move them to their own
// line, similar to test annotations in `loopAdd.ts` and `greaterThan10.ts`.

import range from "../lib/range.ts";
import treeSum from "../lib/treeSum.ts";

export default function main(io: Summon.IO) {
  const N = io.inputPublic('N', summon.number());

  const ballots = range(0, N).map(
    i => io.input(`party${i}`, `ballot${i}`, summon.bool())
  );

  io.outputPublic('result', impl(ballots));
}

function impl(ballots: boolean[]) {
  const sum = treeSum(ballots, b => b ? 1 : 0);
  return sum * 3 >= ballots.length * 2;
}
