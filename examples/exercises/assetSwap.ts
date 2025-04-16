// Asset Swap.
//
// Slow negotiations getting you down? Skip to the end using MPC!
//
// Suppose two people have assets each that they are interested in swapping. These assets could be
// anything from trading cards, to fantasy football players, to company subsidiaries. Although each
// person knows how much they value each asset, they are not incentivized to share this information:
// "Oh that card is only worth $2 to you? Here's $2.01."
//
// To solve this, we can calculate the ideal swap in MPC without revealing any of this sensitive
// information. The MPC simply finds the allocation of assets to each party that maximizes the
// resulting portfolio for the least satisfied party. This means both parties must come out in an
// equal or better position than they started with, since leaving the allocations as-is is a valid
// option.
//
// Each party submits valuations both for their own assets and the assets for the other party.
//
// The circuit should output a number for each asset indicating which party should receive it,
// even if it's going back to the same party. For example:
//
//  No assets swapped:  {
//    party0Asset0: 0,
//    party0Asset1: 0,
//    party0Asset2: 0,
//    party0Asset3: 0,
// 
//    party1Asset0: 1,
//    party1Asset1: 1,
//    party1Asset2: 1,
//    party1Asset3: 1,
//  }
//
//  All assets swapped: {
//    party0Asset0: 1,
//    party0Asset1: 1,
//    party0Asset2: 1,
//    party0Asset3: 1,
//
//    party1Asset0: 0,
//    party1Asset1: 0,
//    party1Asset2: 0,
//    party1Asset3: 0,
//  }
//
//  Party 0 swaps everything for party 1's last asset: {
//    party0Asset0: 1,
//    party0Asset1: 1,
//    party0Asset2: 1,
//    party0Asset3: 1,
//
//    party1Asset0: 1,
//    party1Asset1: 1,
//    party1Asset2: 1,
//    party1Asset3: 0,
//  }
//
// Extension: N parties.

export default function main(io: Summon.IO) {
  // TODO
}
