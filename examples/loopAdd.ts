//! test [5] => [15]

const iterations = 3;

export default (io: Summon.IO) => {
  const input = io.input('alice', 'input', summon.number());

  io.outputPublic('result', loopAdd(input));
};

function loopAdd(input: number) {
  let res = 0;

  for (let i = 0; i < iterations; i++) {
    res += input;
  }

  return res;
}
