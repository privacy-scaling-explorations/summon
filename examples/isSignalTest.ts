//! test [1] => [2]

export default (io: Summon.IO) => {
  const x = io.input('alice', 'x', summon.number());

  let count = 0;

  if (summon.isSignal('hello')) {
    count++;
  }
  // 'hello' is not a signal, count: 0

  if (summon.isSignal(count)) {
    count++;
  }
  // `count` is not a signal, count: 0

  if (summon.isSignal(x)) {
    count += x;
  }
  // `x` is a signal, count: x

  if (summon.isSignal(count)) {
    count++;
  }
  // `count` is a signal, count: x + 1
  // (even though it wasn't before)

  io.outputPublic('count', count);
};
