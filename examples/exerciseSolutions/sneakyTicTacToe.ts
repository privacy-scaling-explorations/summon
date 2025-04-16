// Sneaky Tic-Tac-Toe.
//
// Let's add some hidden information to tic-tac-toe to make it more interesting and suitable for
// MPC. It's the same game, but each player also gets a hidden move.
//
// Each player chooses their hidden move at the start of the game and computes:
//  commitment = hash(salt, movePos)
//
// All other moves are public. If a player attempts to play an existing move, including any of the
// hidden moves, they immediately lose the game.
//
// Otherwise, the winner is decided by the usual tic-tac-toe rules.
//  https://en.wikipedia.org/wiki/Tic-tac-toe
//
// This circuit allows players to compute the correct outcome for a single move of the game. The
// circuit should check that hash(salt, hiddenPos) of each player is equal to their commitment,
// and failure to do so should give victory to the opponent.
//
// Valid moves are 0 for top-left to 8 for bottom-right (see grid layout). An invalid move gives
// victory to the opponent.
//
// Outputs:
//   0 to indicate that play should continue (a valid move was played and nothing happened yet)
//   1 to indicate that player 1 has won
//   2 to indicate that player 2 has won
//  13 to indicate that the hidden moves were equal (which is a null result)
//  14 to indicate anything else was invalid / inconsistent
//     (generally though, the circuit behavior for invalid public inputs is not important - these
//     should be handled outside MPC)

// hash(12345, 0..=8) => [
//   14798714262957021688, // 0
//    4282439475961790587, // 1
//   14939908423030234198, // 2
//   12418860018404658317, // 3
//   16796124678332949243, // 4
//   15249901663317160695, // 5
//    1108717621045500558, // 6
//   14095952657503581013, // 7
//    2638573472227958764, // 8
// ]

// //! test [0, 1, 0, 0, 0, 0, 0, 0, 0, 14798714262957021688, 16796124678332949243, 1, 12345, 0, 12345, 4, 2] => [1]

import hash from '../lib/hash.ts';

export default function main(
  // shared public inputs
  grid0: number, // layout:
  grid1: number, // grid0 grid1 grid2
  grid2: number, // grid3 grid4 grid5
  grid3: number, // grid6 grid7 grid8
  grid4: number,
  grid5: number, // for gridN:
  grid6: number, //  0 means empty
  grid7: number, //  1 means player 1 has played there
  grid8: number, //  2 means player 2 has played there
  player1Commitment: number, // should equal hash(player1Salt, player1HiddenPos)
  player2Commitment: number, // should equal hash(player2Salt, player2HiddenPos)
  currentPlayer: number, // 1 for player 1's turn, 2 for player 2's turn

  // player 1
  player1Salt: number,
  player1HiddenPos: number, // 0 for top-left, 8 for bottom-right (see grid layout)

  // player 2
  player2Salt: number,
  player2HiddenPos: number, // 0 for top-left, 8 for bottom-right (see grid layout)

  // current player
  movePos: number, // 0 for top-left, 8 for bottom-right (see grid layout)
) {
  if (hash(player1Salt, player1HiddenPos) !== player1Commitment) {
    return 2;
  }

  if (hash(player2Salt, player2HiddenPos) !== player2Commitment) {
    return 1;
  }

  if (player1HiddenPos === player2HiddenPos) {
    return 13;
  }

  let grid = [
    grid0,
    grid1,
    grid2,
    grid3,
    grid4,
    grid5,
    grid6,
    grid7,
    grid8,
  ];

  let oldValue: number;

  grid = slowUpdateArray(grid, player1HiddenPos, 1).result;
  grid = slowUpdateArray(grid, player2HiddenPos, 2).result;

  ({
    result: grid,
    oldValue,
  } = slowUpdateArray(grid, movePos, currentPlayer));

  if (oldValue !== 0) {
    return currentPlayer === 1 ? 2 : 1;
  }

  const g = makeGridLookup(grid);

  for (let i = 0; i < 3; i++) {
    if (isTriple(g(i, 0), g(i, 1), g(i, 2))) {
      return g(i, 0);
    }

    if (isTriple(g(0, i), g(1, i), g(2, i))) {
      return g(0, i);
    }
  }

  if (isTriple(g(0, 0), g(1, 1), g(2, 2))) {
    return g(0, 0);
  }

  if (isTriple(g(2, 0), g(1, 1), g(0, 2))) {
    return g(2, 0);
  }

  return 0;
}

function isTriple(a: number, b: number, c: number) {
  return (
    a !== 0 &&
    a === b &&
    b === c
  );
}

function makeGridLookup(grid: number[]) {
  return (i: number, j: number) => grid[3 * i + j];
}

function slowUpdateArray(a: number[], i: number, value: number): {
  result: number[],
  oldValue: number,
} {
  let result: number[] = [];
  let oldValue = 0;

  for (let j = 0; j < a.length; j++) {
    if (i === j) {
      result.push(value);
      oldValue = a[j];
    } else {
      result.push(a[j]);
    }
  }

  return { result, oldValue };
}
