// Asset Swap.
//
// Slow negotiations getting you down? Skip to the end using MPC!
//
// Suppose two people have assets that they are interested in swapping. These assets could be
// anything from trading cards, to fantasy football players, to company subsidiaries. Although each
// person knows how much they value each asset, they are not incentivized to share this information:
// "Oh that card is only worth $2 to you? Here's $2.01."
//
// To solve this, we can calculate an ideal swap in MPC without revealing any of this sensitive
// information:
// - Start with the initial allocation
// - Iterate through all alternative allocations
// - Switch to the alternative if it is mutually acceptable (both parties consider it equal or better)
//
// Each party submits valuations both for their own assets and the assets for the other party:
//
// {
//   // party_0's valuations of their own assets
//   party_0_asset_0_0_value: 10,
//   party_0_asset_0_1_value: 10,
//   party_0_asset_0_2_value: 10,
//   //            | ^ index of asset for that owner
//   //            ^ original owner of asset
//   //            party_0 brings 3 assets to the swap: asset_0_0, asset_0_1, asset_0_2
//
//   // party_0's valuations of party_1's assets
//   party_0_asset_1_0_value: 0,
//   party_0_asset_1_1_value: 0,
//   //            party_1 brings 2 assets to the swap: asset_1_0, asset_1_1
//
//   // party_1's valuations of party_0's assets
//   party_1_asset_0_0_value: 0,
//   party_1_asset_0_1_value: 0,
//   party_1_asset_0_2_value: 0,
//
//   // party_1's valuations of their own assets
//   party_1_asset_1_0_value: 10,
//   party_1_asset_1_1_value: 10,
// }
//
// In this example, each party values their own assets and does not value the other party's assets.
// This should result in each asset being allocated to its original owner:
//
// {
//   asset_0_0: 0,
//   asset_0_1: 0,
//   asset_0_2: 0,
//   asset_1_0: 1,
//   asset_1_1: 1,
// }
//
//! test { n0: 3, n1: 2 } [10, 10, 10, 0, 0, 0, 0, 0, 10, 10] => [0, 0, 0, 1, 1]
//
// In this scenario, party_0 brings a single asset and party_1 brings 5 assets:
//
// {
//   // party_0's valuations of their own assets
//   party_0_asset_0_0_value: 100,
//
//   // party_0's valuations of party_1's assets
//   party_0_asset_1_0_value: 30,
//   party_0_asset_1_1_value: 30,
//   party_0_asset_1_2_value: 30,
//   party_0_asset_1_3_value: 30,
//   party_0_asset_1_4_value: 30,
//
//     // With these valuations, party_0 is willing to swap their asset for any 4 of party_1's
//     // assets (or all 5).
//
//   // party_1's valuations of party_0's assets
//   party_1_asset_0_0_value: 1000,
//
//   // party_1's valuations of their own assets
//   party_1_asset_1_0_value: 6,
//   party_1_asset_1_1_value: 8,
//   party_1_asset_1_2_value: 3,
//   party_1_asset_1_3_value: 5,
//   party_1_asset_1_4_value: 1,
//
//     // party_1 intensely prefers party_0's asset. Since party_0 is equally happy with any 4 of
//     // party_1's assets, if the circuit picks one of these scenarios, it should pick the 4 assets
//     // that party_1 is most willing to lose.
//
//     // Depending on the iteration order, the circuit could also swap all 5 of party_1's assets
//     // since that is mutually preferred to the original allocation. If it sees this allocation
//     // first, then it will stay with that allocation, because the other allocations aren't
//     // *mutually* preferred once that allocation becomes the leader.
//
//     // Do not disambiguate based on an attempt at total valuation. Valuations from different
//     // parties are not comparable. The total valuation is not knowable.
// }
//
// Example correct output:
//
// {
//   asset_0_0: 1, // party_0 gives their single valuable asset to party_0
//   asset_1_0: 0, // party_1 gives this asset to party_0
//   asset_1_1: 1, // party_1 keeps this asset (their most valued original asset)
//   asset_1_2: 0, // party_1 gives this asset to party_0
//   asset_1_3: 0, // party_1 gives this asset to party_0
//   asset_1_4: 0, // party_1 gives this asset to party_0
// }
//
// Since there are two valid solutions, one of the following should pass with `cargo test` once
// uncommented:
//! test { n0: 1, n1: 5 } [100, 30, 30, 30, 30, 30, 1000, 6, 8, 3, 5, 1] => [1, 0, 1, 0, 0, 0]
// //! test { n0: 1, n1: 5 } [100, 30, 30, 30, 30, 30, 1000, 6, 8, 3, 5, 1] => [1, 0, 0, 0, 0, 0]
//
// Extension: N parties.

import range from "../lib/range.ts";

type Score = [number, number];

export default function main(io: Summon.IO) {
  const n0 = io.inputPublic('n0', summon.number());
  const n1 = io.inputPublic('n1', summon.number());

  const assetLabels = [
    ...range(0, n0).map((i) => `asset_0_${i}`),
    ...range(0, n1).map((i) => `asset_1_${i}`),
  ];

  const party0Valuations = assetLabels.map(
    (label) => io.input('party_0', `party_0_${label}_value`, summon.number()),
  );

  const party1Valuations = assetLabels.map(
    (label) => io.input('party_1', `party_1_${label}_value`, summon.number()),
  );

  const partyValuations = [party0Valuations, party1Valuations];

  const initialAlloc = [
    ...range(0, n0).map(() => 0),
    ...range(0, n1).map(() => 1),
  ];

  const allocs = enumerateRelativeAllocations(initialAlloc);

  if (allocs[0] !== initialAlloc) {
    throw new Error("Initial allocation is not the first allocation");
  }

  let leader = initialAlloc;

  // Enhancement 1: Shuffle these
  // Enhancement 2: Put allocs closer to the initial allocation first (shuffle equal distances)
  // Enhancement 3: Minimize depth using a tree while preserving behavior
  const alternativeAllocs = allocs.slice(1);

  function scoreAlloc(alloc: number[]) {
    let score: Score = [0, 0];

    for (const [i, a] of alloc.entries()) {
      score[a] += partyValuations[a][i];
    }

    return score;
  }

  let leaderScore = scoreAlloc(leader);

  for (const candidate of alternativeAllocs) {
    const candidateScore = scoreAlloc(candidate);

    if (isMutualTransition(leaderScore, candidateScore)) {
      leader = candidate;
      leaderScore = candidateScore;
    }
  }

  for (const i of range(0, n0 + n1)) {
    io.outputPublic(assetLabels[i], leader[i]);
  }
}

function enumerateAllocations(nAssets: number) {
  let allocations: number[][] = [[]];

  for (let i = 0; i < nAssets; i++) {
    allocations = [
      ...allocations.map((alloc) => [...alloc, 0]),
      ...allocations.map((alloc) => [...alloc, 1]),
    ];
  }

  return allocations;
}

function enumerateRelativeAllocations(initialAlloc: number[]) {
  const nAssets = initialAlloc.length;
  const allocations = enumerateAllocations(nAssets);

  return allocations.map((alloc) =>
    range(0, nAssets).map((i) => (initialAlloc[i] + alloc[i]) % 2),
  );
}

function isMutualTransition(leaderScore: Score, candidateScore: Score) {
  return (
    candidateScore[0] >= leaderScore[0] && candidateScore[1] >= leaderScore[1]
  );
}
