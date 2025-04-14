//! test [10] => [0]
//! test [11] => [10]

export default function main(_io: Summon.IO, x: number) {
  if (x > 10) {
    return 10;
  }

  return 0;
}
