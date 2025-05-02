// Approval Voting.
//
// About: https://www.youtube.com/watch?v=orybDrUj4vA
//
// A simple voting system allowing each participant to vote based on whether they approve of each
// option. Voters are allowed to approve of multiple options. The result is the option with the
// most approvals.
//
// Example:
//! test { nParties: 6, nOptions: 3 } [ true,  true, false,    true,  true, false,   false,  true, false,   false,  true,  true,   false,  true,  true,   false,  true,  true] => [1]
//! test { nParties: 3, nOptions: 5 } [false, false, false,  true, false,    true, false, false,  true, false,    true, false, false,  true, false] => [3]

import range from "../lib/range.ts";
import treeReduce from "../lib/treeReduce.ts";
import treeSum from "../lib/treeSum.ts";

export default function main(io: Summon.IO) {
  const nParties = io.inputPublic('nParties', summon.number());
  const nOptions = io.inputPublic('nOptions', summon.number());
  const ballots = range(0, nParties).map(i => inputBallot(io, i, nOptions));

  io.outputPublic('result', impl(nOptions, ballots));
}

function inputBallot(io: Summon.IO, partyIndex: number, nOptions: number): boolean[] {
  return range(0, nOptions).map(
    i => io.input(`party${partyIndex}`, `party${partyIndex}Option${i}`, summon.bool())
  );
}

function impl(nOptions: number, ballots: boolean[][]) {
  const ballotSums = range(0, nOptions).map(
    optionIndex => ({
      optionIndex,
      count: treeSum(ballots, b => b[optionIndex] ? 1 : 0),
    }),
  );

  const winningSum = treeReduce(
    ballotSums,
    (a, b) => a.count > b.count ? a : b,
  );

  return winningSum.optionIndex;
}
