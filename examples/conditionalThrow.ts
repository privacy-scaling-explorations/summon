//! test [10] => [0]
//! test [11] => [10]

export default (io: Summon.IO) => {
  const x = io.input('alice', 'x', summon.number());

  io.outputPublic('result', test(x));
};

function test(x: number) {
  try {
    check(x);
  } catch {
    return 10;
  }

  return 0;
}

function check(x: number) {
  if (x > 10) {
    throw new Error('boom');
  }
}
