import median from "./lib/median.ts";

export default (io: Summon.IO) => {
  const nParties = io.inputPublic('nParties', summon.number());
  let x: number[] = [];

  for (let i = 0; i < nParties; i++) {
    x.push(io.input(`party${i}`, `x${i}`, summon.number()));
  }

  io.outputPublic('median', median(x));
}
