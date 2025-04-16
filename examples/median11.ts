//! test [4, 32, 8, 10, 50, 5, 20, 25, 38, 58, 93] => [25]

// Note: This example hard codes the number of parties to 11 for the sake of easy testing.
// You can also take nParties as a public input so you can generate a circuit of the required size
// using the same Summon program. See medianCircuit.ts.

import median from "./lib/median.ts";

export default function median11(io: Summon.IO) {
  let x: number[] = [];

  for (let i = 0; i < 11; i++) {
    x.push(io.input(`party${i}`, `x${i}`, summon.number()));
  }

  io.outputPublic('median', median(x));
}
