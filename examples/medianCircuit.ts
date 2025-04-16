//! test { N: 1 } [4] => [4]
//! test { N: 2 } [4, 32] => [18]
//! test { N: 3 } [4, 32, 8] => [8]
//! test { N: 11 } [4, 32, 8, 10, 50, 5, 20, 25, 38, 58, 93] => [25]

import median from "./lib/median.ts";

export default (io: Summon.IO) => {
  const N = io.inputPublic('N', summon.number());
  let x: number[] = [];

  for (let i = 0; i < N; i++) {
    x.push(io.input(`party${i}`, `x${i}`, summon.number()));
  }

  io.outputPublic('median', median(x));
}
