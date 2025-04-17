//! test [1, 2] => [5, 0]

import outputArrayWorkaround from "./lib/outputArrayWorkaround.ts";

export default (io: Summon.IO) => {
  const a = io.input('alice', 'a', summon.number());
  const b = io.input('bob', 'b', summon.number());

  outputArrayWorkaround(io, 'result', test(a, b));
};

function test(a: number, b: number) {
  if (a === b) {
    return [12, a ? 0 : 0];
  }

  return [5, 0];
}
