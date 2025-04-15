/**
 * In future, you will be able to write `io.outputPublic(name, array)` directly, which will result
 * in getting that actual array out of the circuit.
 *
 * Right now, only numbers are supported, so we workaround this limitation by outputting name[0],
 * name[1], etc for each element.
 */
export default function outputArrayWorkaround(io: Summon.IO, name: string, array: number[]) {
  for (const [i, value] of array.entries()) {
    io.outputPublic(`${name}[${i}]`, value);
  }
}
