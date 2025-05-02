//! test [false, false] => [false]
//! test [false,  true] => [false]
//! test [ true, false] => [false]
//! test [ true,  true] => [ true]

export default (io: Summon.IO) => {
  const x = io.input('alice', 'x', summon.bool());
  const y = io.input('alice', 'y', summon.bool());

  io.outputPublic('result', x && y);
}
