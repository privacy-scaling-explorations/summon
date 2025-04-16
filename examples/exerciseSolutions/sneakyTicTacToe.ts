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
// Moves must be on the grid (0 <= i,j <= 2). Any invalid move gives victory to the opponent.
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
//                        i, j
//   8893086672312824, // 0, 0
//   4019829959819387, // 0, 1
//   5972058669669462, // 0, 2
//   6939445371571341, // 1, 0
//   6705267495740155, // 1, 1
//    713325040661239, // 1, 2
//    832112712358542, // 2, 0
//   8693023088669525, // 2, 1
//   8471289843589100, // 2, 2
// ]

// Player 1 wins by completing the first row.
//
//  (1)  1   1*
//
//   0  (2)  0
//
//   0   0   0
//
//! test { grid[0][0]: 0, grid[0][1]: 1, grid[0][2]: 0, grid[1][0]: 0, grid[1][1]: 0, grid[1][2]: 0, grid[2][0]: 0, grid[2][1]: 0, grid[2][2]: 0, player1Commitment: 8893086672312824, player2Commitment: 6705267495740155, currentPlayer: 1, moveI: 0, moveJ: 2 } [12345, 0, 0, 12345, 1, 1] => [1]

// Player 2 wins because player 1's hidden move does not match their commitment.
//
//  (1)  1   1*
//
//   0  (2)  0
//
//   0   0   0
//
//! test { grid[0][0]: 0, grid[0][1]: 1, grid[0][2]: 0, grid[1][0]: 0, grid[1][1]: 0, grid[1][2]: 0, grid[2][0]: 0, grid[2][1]: 0, grid[2][2]: 0, player1Commitment: 987, player2Commitment: 6705267495740155, currentPlayer: 1, moveI: 0, moveJ: 2 } [12345, 0, 0, 12345, 1, 1] => [2]

// Play continues (nothing went wrong, no victory occurred).
//
//  (1)  1   0
//
//   0  (2)  1*
//
//   0   0   0
//
//! test { grid[0][0]: 0, grid[0][1]: 1, grid[0][2]: 0, grid[1][0]: 0, grid[1][1]: 0, grid[1][2]: 0, grid[2][0]: 0, grid[2][1]: 0, grid[2][2]: 0, player1Commitment: 8893086672312824, player2Commitment: 6705267495740155, currentPlayer: 1, moveI: 1, moveJ: 2 } [12345, 0, 0, 12345, 1, 1] => [0]

// Player 1 wins because player 2's hidden move does not match their commitment.
//
//  (1)  1   0
//
//   0  (2)  1*
//
//   0   0   0
//
//! test { grid[0][0]: 0, grid[0][1]: 1, grid[0][2]: 0, grid[1][0]: 0, grid[1][1]: 0, grid[1][2]: 0, grid[2][0]: 0, grid[2][1]: 0, grid[2][2]: 0, player1Commitment: 8893086672312824, player2Commitment: 987, currentPlayer: 1, moveI: 1, moveJ: 2 } [12345, 0, 0, 12345, 1, 1] => [1]

import hash from '../lib/hash.ts';
import range from '../lib/range.ts';

export default function main(io: Summon.IO) {
  let grid = inputGrid(io);

  // should equal hash(player1Salt, player1HiddenPos)
  const player1Commitment = io.inputPublic('player1Commitment', summon.number());

  // should equal hash(player2Salt, player2HiddenPos)
  const player2Commitment = io.inputPublic('player2Commitment', summon.number());

  const player1Salt = io.input('player1', 'player1Salt', summon.number());
  const player1HiddenPos = {
    i: io.input('player1', 'player1HiddenI', summon.number()),
    j: io.input('player1', 'player1HiddenJ', summon.number()),
  };

  const player2Salt = io.input('player2', 'player2Salt', summon.number());
  const player2HiddenPos = {
    i: io.input('player2', 'player2HiddenI', summon.number()),
    j: io.input('player2', 'player2HiddenJ', summon.number()),
  };

  const currentPlayer = io.inputPublic('currentPlayer', summon.number());

  if (currentPlayer !== 1 && currentPlayer !== 2) {
    throw new Error('Invalid current player');
  }

  // The current move is determined by the current player, but it's still public. (If this is made
  // into an app, the parties should establish the current player's choice outside the circuit so
  // they can input the same values.)
  const movePos = {
    i: io.inputPublic('moveI', summon.number()),
    j: io.inputPublic('moveJ', summon.number()),
  };

  const result = impl(
    grid,
    player1Commitment,
    player1Salt,
    player1HiddenPos,
    player2Commitment,
    player2Salt,
    player2HiddenPos,
    movePos,
    currentPlayer,
  );

  io.outputPublic('result', result);
}

function impl(
  grid: number[][],
  player1Commitment: number,
  player1Salt: number,
  player1HiddenPos: Pos,
  player2Commitment: number,
  player2Salt: number,
  player2HiddenPos: Pos,
  movePos: Pos,
  currentPlayer: number,
): number {
  const commitmentCheck = checkCommitments(
    player1Commitment,
    player1Salt,
    player1HiddenPos,
    player2Commitment,
    player2Salt,
    player2HiddenPos,
  );

  if (commitmentCheck !== 0) {
    return commitmentCheck;
  }

  if (player1HiddenPos.i === player2HiddenPos.i && player1HiddenPos.j === player2HiddenPos.j) {
    return 13;
  }

  let oldValue: number;

  grid = slowUpdateMatrix(grid, player1HiddenPos, 1).result;
  grid = slowUpdateMatrix(grid, player2HiddenPos, 2).result;

  ({
    result: grid,
    oldValue,
  } = slowUpdateMatrix(grid, movePos, currentPlayer));

  if (oldValue !== 0) {
    return currentPlayer === 1 ? 2 : 1;
  }

  for (const triple of allTriples()) {
    // this triple is only relevant if it contains the current move
    const relevant = triple.some((pos) => pos.i === movePos.i && pos.j === movePos.j);

    if (!relevant) {
      continue;
    }

    if (isCompletedTriple(grid, triple)) {
      return currentPlayer;
    }
  }

  return 0;
}

type Pos = { i: number; j: number; };
type Triple = [Pos, Pos, Pos];

function* allTriples(): Generator<Triple> {
  for (let i = 0; i < 3; i++) {
    // rows
    yield [{ i, j: 0 }, { i, j: 1 }, { i, j: 2 }];

    // columns
    yield [{ i: 0, j: i }, { i: 1, j: i }, { i: 2, j: i }];
  }

  // diagonals
  yield [{ i: 0, j: 0 }, { i: 1, j: 1 }, { i: 2, j: 2 }];
  yield [{ i: 2, j: 0 }, { i: 1, j: 1 }, { i: 0, j: 2 }];
}

function inputGrid(io: Summon.IO): number[][] {
  // 3x3 grid (input names: grid[0][0], .. grid[2][2])
  // in each cell:
  //  0: empty or contains a hidden move
  //  1: player 1 has played there
  //  2: player 2 has played there
  
  return range(0, 3).map(
    (i) => range(0, 3).map(
      (j) => io.inputPublic(`grid[${i}][${j}]`, summon.number()),
    ),
  );
}

function isCompletedTriple(grid: number[][], triple: Triple): boolean {
  const [a, b, c] = triple.map((pos) => grid[pos.i][pos.j]);

  return (
    a !== 0 &&
    a === b &&
    b === c
  );
}

function slowUpdateMatrix(a: number[][], { i, j }: { i: number, j: number }, value: number): {
  result: number[][],
  oldValue: number,
} {
  let result = a;
  let oldValue = 0;

  const rows = a.length;
  const cols = a[0].length;

  for (let ii = 0; ii < rows; ii++) {
    for (let jj = 0; jj < cols; jj++) {
      if (ii === i && jj === j) {
        oldValue = a[ii][jj];
        result[ii][jj] = value;
      }
    }
  }

  return { result, oldValue };
}

function checkCommitments(
  player1Commitment: number,
  player1Salt: number,
  player1HiddenPos: { i: number; j: number; },
  player2Commitment: number,
  player2Salt: number,
  player2HiddenPos: { i: number; j: number; },
): number {
  if (hashPos(player1Salt, player1HiddenPos) !== player1Commitment) {
    return 2;
  }

  if (hashPos(player2Salt, player2HiddenPos) !== player2Commitment) {
    return 1;
  }

  return 0;
}

function hashPos(salt: number, pos: { i: number; j: number; }) {
  return hash(salt, 3 * pos.i + pos.j);
}
