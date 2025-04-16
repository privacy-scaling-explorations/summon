export default function range(start: number, end: number) {
  let result = [];

  for (let i = start; i < end; i++) {
    result.push(i);
  }

  return result;
}
