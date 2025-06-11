// //! test [false, true, true, true, false, false, true, true, false, true, true, true, false, true, false, true, false, true, true, false, true, true, false, true, false, true, true, false, true, true, false, true, false, true, true, false, true, true, true, true, false, true, true, false, true, true, true, false] => [false, false, true, false, true, false, false, false, false, false, false, true, false, true, false, true, true, true, false, false, true, false, true, true, false, false, false, false, false, false, true, false, true, false, true, true, true, false, false, true, false, true, false, true, true, false, true, true, false, true, true, false, true, true, false, true, false, false, false, true, false, true, false, true, false, false, true, true, true, false, false, false, false, false, true, true, true, false, true, true, true, true, true, true, false, true, false, true, false, true, false, true, false, false, false, true, true, true, true, true, false, false, false, false, true, false, false, true, true, false, true, true, false, false, true, true, false, false, true, true, true, true, true, false, false, false, false, false, false, false, false, true, true, false, false, false, false, false, false, false, false, true, true, false, true, false, true, false, true, true, false, true, false, false, true, false, true, true, true, true, false, true, false, false, false, false, true, false, false, false, true, false, false, false, false, true, true, false, true, true, false, false, false, false, false, false, true, true, false, true, false, true, true, false, true, false, false, true, false, true, true, false, false, true, false, false, true, false, true, true, false, false, false, false, false, true, true, false, true, true, true, true, true, false, false, false, false, true, false, true, false, false, false, true, true, false, true, true, false, true, false, true, true, false, true, false, true, false, true, false, false, true, true, false, false, true]
// "summon" => 2815cb02b95b6d15383bf551f09b33e01806ad2f4221b035a592c1be146d6a99
// note: disabled by default because this is by far the slowest test (8s debug, 2s release)

import sha256 from './deps/sha256/mod.ts';

export default (io: Summon.IO) => {
  // 6 characters * 8 bits, so we can encode 'summon'
  const input = range(6 * 8).map(
    i => io.input('alice', `input${i}`, summon.bool()),
  );

  // expected: 2815cb02b95b6d15383bf551f09b33e01806ad2f4221b035a592c1be146d6a99
  // (but encoded as bits)
  const output = sha256(input);

  for (const [i, b] of output.entries()) {
    io.outputPublic(`output${i}`, b);
  }
};

function range(limit: number) {
  let res = [];

  for (let i = 0; i < limit; i++) {
    res.push(i);
  }

  return res;
}
