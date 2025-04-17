export default function hash(salt: number, msg: number): number {
  let [a, b] = [salt, msg];

  for (let i = 0; i < 16; i++) {
    a ^= b;
    b = (b << 11) + (b >> 53) + 123456789012345;
    [a, b] = [b, a];
  }

  // Max safe integer in f64. This is a workaround due to encoding issues.
  const mask = 0b11111111111111111111111111111111111111111111111111111;

  return (a + b) & mask;
}
