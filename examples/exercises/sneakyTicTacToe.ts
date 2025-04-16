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

import hash from '../lib/hash.ts';

export default function main(io: Summon.IO) {
  const grid = inputGrid(io);

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

  // check the commitments are correct
  // check the grid for win conditions
  throw new Error('Implement me');
}

function inputGrid(io: Summon.IO): number[][] {
  // 3x3 grid (input names: grid[0][0], .. grid[2][2])
  // in each cell:
  //  0: empty or contains a hidden move
  //  1: player 1 has played there
  //  2: player 2 has played there
  throw new Error('Implement me');
}
