// Approval Voting.
//
// About: https://www.youtube.com/watch?v=orybDrUj4vA
//
// A simple voting system allowing each participant to vote based on whether they approve of each
// option. Voters are allowed to approve of multiple options. The result is the option with the
// most approvals.
//
// Example:
//  //! test { N: 6 } [true, true, false,   true, true, false,   false, true, false,   false, true, true,   false, true, true,   false, true, true] => [1]
//
// Output meanings:
//   0: Steak Shack
//   1: Burger Barn
//   2: Veggie Villa

import range from "../lib/range.ts";

export default function main(io: Summon.IO) {
  const N = io.inputPublic('N', summon.number());
  const ballots = range(0, N).map(i => inputBallot(io, i));

  io.outputPublic('result', impl(ballots));
}

// Extension: Use another public input to allow any number of options (more restaurants to choose
// from) (use option0, option1, etc, rather than specific names).
type Ballot = {
  steakShack: boolean;
  burgerBarn: boolean;
  veggieVilla: boolean;
};

function inputBallot(io: Summon.IO, partyIndex: number): Ballot {
  return {
    steakShack: io.input(`party${partyIndex}`, `steakShack${partyIndex}`, summon.bool()),
    burgerBarn: io.input(`party${partyIndex}`, `burgerBarn${partyIndex}`, summon.bool()),
    veggieVilla: io.input(`party${partyIndex}`, `veggieVilla${partyIndex}`, summon.bool()),
  };
}

function impl(ballots: Ballot[]): boolean {
  throw new Error('Implement me');
}
