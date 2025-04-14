//! test [1, 2] => [5, 0]

export default function main(_io: Summon.IO, a: number, b: number) {
  if (a === b) {
    return [12, a ? 0 : 0];
  }

  return [5, 0];
}
