// Check for a Supermajority.
//
// Write a circuit which checks whether 2/3 or more ballots approved the motion (supermajority).
// Each ballot is a number, with 0 meaning 'no' and any other number meaning 'yes'.
// The circuit should return 1 to indicate the motion passes and 0 to indicate it fails.
//
// For example:
//! test [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] => [0]
//! test [1, 1, 1, 1, 1, 1, 1, 1, 1, 1] => [1]
//! test [0, 7, 8, 0, 1, 1, 0, 0, 2, 2] => [0]
//! test [0, 7, 8, 0, 1, 1, 0, 3, 2, 2] => [1]
//
// The format above is also used to check circuits with `cargo test`. Simply move them to their own
// line, similar to test annotations in `loopAdd.ts` and `greaterThan10.ts`.

import treeSum from "../lib/treeSum.ts";

export default function main(io: Summon.IO) {
  // 10 inputs are hardcoded for the sake of simplicity.
  // As an extension, you can use `io.publicInput` to get the number of parties at compile time and
  // use that to dynamically gather the inputs. See `medianCircuit.ts` for an example.

  const ballots = [
    io.input('party0', 'ballot0', summon.number()),
    io.input('party1', 'ballot1', summon.number()),
    io.input('party2', 'ballot2', summon.number()),
    io.input('party3', 'ballot3', summon.number()),
    io.input('party4', 'ballot4', summon.number()),
    io.input('party5', 'ballot5', summon.number()),
    io.input('party6', 'ballot6', summon.number()),
    io.input('party7', 'ballot7', summon.number()),
    io.input('party8', 'ballot8', summon.number()),
    io.input('party9', 'ballot9', summon.number()),
  ];

  io.outputPublic('result', impl(ballots) ? 1 : 0);
}

function impl(ballots: number[]) {
  const sum = treeSum(ballots, b => b !== 0 ? 1 : 0);
  return sum * 3 >= ballots.length * 2;
}
