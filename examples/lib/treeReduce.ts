export default function treeReduce<T>(
  items: T[],
  reducer: (a: T, b: T) => T,
): T {
  if (items.length === 0) {
    throw new Error("Cannot reduce an empty array");
  }

  if (items.length === 1) {
    return items[0];
  }

  const mid = Math.floor(items.length / 2);

  return reducer(
    treeReduce(items.slice(0, mid), reducer),
    treeReduce(items.slice(mid), reducer),
  );
}
