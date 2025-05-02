// Check for a Supermajority.
//
// Write a circuit which checks whether 2/3 or more ballots approved the motion (supermajority).
// Each ballot is a number, with 0 meaning 'no' and any other number meaning 'yes'.
// The circuit should return 1 to indicate the motion passes and 0 to indicate it fails.
//
// For example:
//  //! test [false, false, false, false, false, false, false, false, false, false] => [false]
//  //! test [ true,  true,  true,  true,  true,  true,  true,  true,  true,  true] => [ true]
//  //! test [false,  true,  true, false,  true,  true, false, false,  true,  true] => [false]
//  //! test [false,  true,  true, false,  true,  true, false,  true,  true,  true] => [ true]
//
// The format above is also used to check circuits with `cargo test`. Simply move them to their own
// line, similar to test annotations in `loopAdd.ts` and `greaterThan10.ts`.

export default function main(io: Summon.IO) {
  // 10 inputs are hardcoded for the sake of simplicity.
  // As an extension, you can use `io.publicInput` to get the number of parties at compile time and
  // use that to dynamically gather the inputs. See `medianCircuit.ts` for an example.

  const ballots = [
    io.input('party0', 'ballot0', summon.bool()),
    io.input('party1', 'ballot1', summon.bool()),
    io.input('party2', 'ballot2', summon.bool()),
    io.input('party3', 'ballot3', summon.bool()),
    io.input('party4', 'ballot4', summon.bool()),
    io.input('party5', 'ballot5', summon.bool()),
    io.input('party6', 'ballot6', summon.bool()),
    io.input('party7', 'ballot7', summon.bool()),
    io.input('party8', 'ballot8', summon.bool()),
    io.input('party9', 'ballot9', summon.bool()),
  ];

  io.outputPublic('result', impl(ballots) ? 1 : 0);
}

function impl(ballots: boolean[]): boolean {
  // TODO: Return true iff 2/3 or more ballots are non-zero. (In the resulting circuit, true will
  // be converted to 1 and false will be converted to 0.)
  throw new Error('Implement me');
}
