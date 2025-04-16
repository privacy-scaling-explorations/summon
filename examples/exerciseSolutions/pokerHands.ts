// Poker Hands.
//
// Write a circuit to compare two poker hands.
//
// Using this circuit directly you could simply claim you have a royal flush, but in practice you
// would incorporate this into a larger circuit.
//
// Poker hands and their rankings are described here:
//  https://en.wikipedia.org/wiki/List_of_poker_hands
//
// Cards are encoded like this:
//   0-12: Hearts Ace to King
//  13-25: Clubs Ace to King
//  26-38: Diamonds Ace to King
//  39-51: Spades Ace to King
//
// Output:
//  0: hands equal
//  1: player1's hand wins
//  2: player2's hand wins

// //! test [0, 1, 2, 3, 4,   1, 2, 3, 5, 5] => [1]

import batcherSort from "../lib/batcherSort.ts";

const categories = {
  fiveOfAKind: 9,
  straightFlush: 8,
  fourOfAKind: 7,
  fullHouse: 6,
  flush: 5,
  straight: 4,
  threeOfAKind: 3,
  twoPair: 2,
  pair: 1,
  highCard: 0,
};

export default function main(
  player1Card1: number,
  player1Card2: number,
  player1Card3: number,
  player1Card4: number,
  player1Card5: number,

  player2Card1: number,
  player2Card2: number,
  player2Card3: number,
  player2Card4: number,
  player2Card5: number,
) {
  const player1Class = classifyPokerHand([
    player1Card1,
    player1Card2,
    player1Card3,
    player1Card4,
    player1Card5,
  ]);

  const player2Class = classifyPokerHand([
    player2Card1,
    player2Card2,
    player2Card3,
    player2Card4,
    player2Card5,
  ]);

  return lexOrder(player1Class, player2Class);
}

export function classifyPokerHand(cards: number[]): number[] {
  let cardValues = batcherSort(cards.map(c => (c + 12) % 13 + 1));
  const [straight, straightCardValues] = isStraight(cardValues);

  if (straight) {
    cardValues = straightCardValues;
  }

  const cardValuesDesc = [
    cardValues[4],
    cardValues[3],
    cardValues[2],
    cardValues[1],
    cardValues[0],
  ];

  const suits = cards.map(c => c / 13);

  const flush = (
    suits[0] === suits[1] &&
    (suits[1] === suits[2] && suits[3] === suits[4]) &&
    suits[2] === suits[3]
  );

  const nKindClass = classifyNKind(cardValues);
  const cond = straight && flush;

  if (cond) {
    if (nKindClass[0] > categories.straightFlush) {
      return nKindClass;
    }

    return [categories.straightFlush, ...cardValuesDesc];
  }

  if (flush) {
    if (nKindClass[0] > categories.flush) {
      return nKindClass;
    }

    return [categories.flush, ...cardValuesDesc];
  }

  if (straight) {
    if (nKindClass[0] > categories.straight) {
      return nKindClass;
    }

    return [categories.straight, ...cardValuesDesc];
  }

  return nKindClass;
}

function isStraight(cardValues: number[]): [boolean, number[]] {
  for (let i = 0; i <= 2; i++) {
    if (cardValues[i + 1] !== cardValues[i] + 1) {
      return [false, cardValues];
    }
  }

  if (cardValues[4] === 13) { // ace
    if (cardValues[3] === 12) {
      return [true, [9, 10, 11, 12, 13]];
    }

    if (cardValues[0] === 1) { // ace is 0 so 2 is 1 😅
      return [true, [0, 1, 2, 3, 4]];
    }
  }

  return [cardValues[1] === cardValues[0] + 1, cardValues];
}

function classifyNKind(cardValues: number[]) {
  const eq01 = cardValues[0] === cardValues[1];
  const eq12 = cardValues[1] === cardValues[2];
  const eq23 = cardValues[2] === cardValues[3];
  const eq34 = cardValues[3] === cardValues[4];

  const eq012 = eq01 && eq12;
  const eqAll = eq012 && eq34 && eq23;

  if (eqAll) {
    return [categories.fiveOfAKind, ...cardValues];
  }

  const eq0123 = eq012 && eq23;

  if (eq0123) {
    return [categories.fourOfAKind, ...cardValues];
  }

  const eq1234 = eq12 && eq34 && eq23;

  if (eq1234) {
    return [categories.fourOfAKind, ...cardValues.slice(1), cardValues[0]];
  }

  if (eq012) {
    return threeOfAKindOrFullHouse([
      ...cardValues.slice(0, 3),
      cardValues[4],
      cardValues[3],
    ]);
  }

  if (eq12 && eq23) {
    return threeOfAKindOrFullHouse([
      ...cardValues.slice(1, 4),
      cardValues[4],
      cardValues[0],
    ]);
  }

  if (eq23 && eq34) {
    return threeOfAKindOrFullHouse([
      ...cardValues.slice(2),
      cardValues[1],
      cardValues[0],
    ]);
  }

  if (eq01 && eq23) {
    const [lower, higher] = sort2(cardValues[0], cardValues[2]);
    return [categories.twoPair, higher, higher, lower, lower, cardValues[4]];
  }

  if (eq01 && eq34) {
    const [lower, higher] = sort2(cardValues[0], cardValues[3]);
    return [categories.twoPair, higher, higher, lower, lower, cardValues[2]];
  }

  if (eq12 && eq34) {
    const [lower, higher] = sort2(cardValues[1], cardValues[3]);
    return [categories.twoPair, higher, higher, lower, lower, cardValues[0]];
  }

  const pairConditions = [eq01, eq12, eq23, eq34];

  for (let i = 0; i < pairConditions.length; i++) {
    if (pairConditions[i]) {
      return [
        categories.pair,
        cardValues[i],
        cardValues[i + 1],
        ...cardValues.slice(i + 2),
        ...cardValues.slice(0, i),
      ];
    }
  }

  return [
    categories.highCard,
    cardValues[4],
    cardValues[3],
    cardValues[2],
    cardValues[1],
    cardValues[0],
  ];
}

function threeOfAKindOrFullHouse(cardValues: number[]): number[] {
  const category = cardValues[3] === cardValues[4]
    ? categories.fullHouse
    : categories.threeOfAKind;

  return [category, ...cardValues];
}

function lexOrder(a: number[], b: number[]) {
  for (let i = 0; i < a.length; i++) {
    if (a[i] > b[i]) {
      return 1;
    }

    if (b[i] > a[i]) {
      return 2;
    }
  }

  return 0;
}

function sort2(a: number, b: number): [number, number] {
  return a < b ? [a, b] : [b, a];
}
