// Asset Swap.
//
// Slow negotiations getting you down? Skip to the end using MPC!
//
// Suppose two people have 4 assets each that they are interested in swapping. These assets could
// be anything from trading cards, to fantasy football players, to company subsidiaries. Although
// each person knows how much they value each asset, they are not incentivized to share this
// information: "Oh that card is only worth $2 to you? Here's $2.01."
//
// To solve this, we can calculate the ideal swap in MPC without revealing any of this sensitive
// information. The MPC simply finds the allocation of assets to each party that maximizes the
// resulting portfolio for the least satisfied party. This means both parties must come out in an
// equal or better position than they started with, since leaving the allocations as-is is a valid
// option.
//
// Each party submits valuations both for their own assets and the assets for the other party.
// Party 1 owns assets 1 to 4 and party 2 owns assets 5 to 8.
//
// The circuit should output 8 numbers indicating which party should receive each asset, for
// example:
//  No assets swapped:  [1, 1, 1, 1, 2, 2, 2, 2]
//  All assets swapped: [2, 2, 2, 2, 1, 1, 1, 1]
//
//  Party 1 swaps everything for asset 8:
//                      [2, 1, 1, 1, 1, 1, 1, 1]

export default function main(
  party1Asset1Valuation: number,
  party1Asset2Valuation: number,
  party1Asset3Valuation: number,
  party1Asset4Valuation: number,
  party1Asset5Valuation: number,
  party1Asset6Valuation: number,
  party1Asset7Valuation: number,
  party1Asset8Valuation: number,

  party2Asset1Valuation: number,
  party2Asset2Valuation: number,
  party2Asset3Valuation: number,
  party2Asset4Valuation: number,
  party2Asset5Valuation: number,
  party2Asset6Valuation: number,
  party2Asset7Valuation: number,
  party2Asset8Valuation: number,
) {
  const party1Valuations = [
    party1Asset1Valuation,
    party1Asset2Valuation,
    party1Asset3Valuation,
    party1Asset4Valuation,
    party1Asset5Valuation,
    party1Asset6Valuation,
    party1Asset7Valuation,
    party1Asset8Valuation,
  ];

  const party2Valuations = [
    party2Asset1Valuation,
    party2Asset2Valuation,
    party2Asset3Valuation,
    party2Asset4Valuation,
    party2Asset5Valuation,
    party2Asset6Valuation,
    party2Asset7Valuation,
    party2Asset8Valuation,
  ];

  return findBest([], 0, party1Valuations, 0, party2Valuations).alloc;
}

type BestOutput = {
  alloc: number[],
  bestMin: number,
};

function findBest(
  alloc: number[],
  party1Total: number,
  party1Valuations: number[],
  party2Total: number,
  party2Valuations: number[]
): BestOutput {
  if (party1Valuations.length === 0) {
    return { alloc, bestMin: min(party1Total, party2Total) };
  }

  const currentValuations = {
    party1: party1Valuations[0],
    party2: party2Valuations[0],
  };

  const nextValuations = {
    party1: party1Valuations.slice(1),
    party2: party2Valuations.slice(1),
  };

  const left = findBest(
    [...alloc, 1],
    party1Total + currentValuations.party1,
    nextValuations.party1,
    party2Total,
    nextValuations.party2,
  );

  const right = findBest(
    [...alloc, 2],
    party1Total,
    nextValuations.party1,
    party2Total + currentValuations.party2,
    nextValuations.party2,
  );

  return left.bestMin > right.bestMin ? left : right;
}

function min(a: number, b: number) {
  return a < b ? a : b;
}
