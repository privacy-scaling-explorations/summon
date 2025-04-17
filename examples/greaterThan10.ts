//! test [10] => [0]
//! test [11] => [10]

export default (io: Summon.IO) => {
  const x = io.input('alice', 'x', summon.number());

  io.outputPublic('result', greaterThan10(x));
};

function greaterThan10(x: number) {
  if (x > 10) {
    return 10;
  }

  return 0;
}
