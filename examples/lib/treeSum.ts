// Note: This could be implemented using `treeReduce` but we hardcode this version because it's
// common and easier to understand without the extra abstraction.

export default function treeSum<T>(items: T[], mapper: (x: T) => number): number {
  if (items.length <= 2) {
    let sum = 0;

    for (const it of items) {
      sum += mapper(it);
    }

    return sum;
  }

  const mid = Math.floor(items.length / 2);

  return treeSum(items.slice(0, mid), mapper) + treeSum(items.slice(mid), mapper);
}
