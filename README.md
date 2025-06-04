# Summon

_A language for collaboratively summoning computations._

Based on [ValueScript](https://github.com/voltrevo/ValueScript).

## Setup

```sh
cargo build
export PATH="$PATH:$PWD/target/debug"
```

## Usage

```sh
summonc main.ts
```

This will generate the circuit in
[bristol format](https://nigelsmart.github.io/MPC-Circuits/) at
`output/circuit.txt` and a description of the inputs, outputs, and constants at
`output/circuit_info.json`.

You can also produce boolean circuits by adding `--boolify-width 16`. (See
[boolify](https://github.com/privacy-scaling-explorations/boolify) for more
about boolean circuits.)

### TypeScript Bindings

Summon also has TypeScript bindings! This means you can create an MPC app from
end to end in TypeScript. See
[`summon-ts`](https://github.com/privacy-scaling-explorations/summon-ts).

## Example

```ts
// examples/loopAdd.ts

const iterations = 3;

export default (io: Summon.IO) => {
  const input = io.input("alice", "input", summon.number());

  let res = 0;

  for (let i = 0; i < iterations; i++) {
    res += input;
  }

  io.outputPublic("res", res);
};
```

```sh
summonc examples/loopAdd.ts
```

```
# output/circuit.txt

2 3
1 1
1 1

2 1 0 0 1 AAdd
2 1 1 0 2 AAdd
```

```jsonc
// output/circuit_info.json

{
  "constants": [],
  "inputs": [
    {
      "name": "input",
      "type": "number",
      "address": 0,
      "width": 1
    }
  ],
  "outputs": [
    {
      "name": "res",
      "type": "number",
      "address": 2,
      "width": 1
    }
  ]
}
```

```jsonc
// output/mpc_settings.json

[
  {
    "name": "alice",
    "inputs": ["input"],
    "outputs": ["res"]
  }
]
```

## Signal-Dependent Branching

Building a circuit from a program with a fixed path is relatively
straightforward. The real power of Summon is its ability to handle
signal-dependent branches - where the program follows a different path depending
on the input. For example:

```ts
// examples/greaterThan10.ts

export default (io: Summon.IO) => {
  const x = io.input("alice", "x", summon.number());

  io.outputPublic("result", greaterThan10(x));
};

function greaterThan10(x: number) {
  if (x > 10) {
    return 10;
  }

  return 0;
}
```

```
2 1 0 1 2 AGt
2 1 2 1 3 AMul
```

Above, the constant 10 is used for wire 1, so the circuit is
`output = (x > 10) * 10`.

Summon can also handle more complex branching, so you can use loops and even
things like `continue`, `break`, and `switch`. You can also conditionally throw
exceptions as long as you catch them.

To achieve this, Summon has a general solution to handle any conditional jump
instruction. A conditional jump generates a new evaluation branch, and each
branch tracks a multiplier signal. Summon dynamically manages these branches and
merges them when they reach the same location.

However, it is easy to write programs which branch indefinitely and never
consolidate into a single fixed circuit. Programs like this become infinite
loops:

```ts
for (let i = 0; i < input; i++) {
  sum += i;
}
```

A traditional runtime can terminate shortly after `i` reaches `input`, but
because `input` isn't known during compilation, Summon will get stuck in a loop
as it adds more and more circuitry to handle larger and larger values of `input`
forever.

## Limitations

- You can't use a signal as an array index
- Compile-time number operations use f64
- Math functions don't work with signals
  - You have to write your own versions of `Math.min`, `Math.max`, etc

## Exercises

If you'd like to try your hand at Summon but you're not sure where to start, I
have prepared some exercises you might find interesting:

- [Check Supermajority](./examples/exercises/checkSuperMajority.ts)
- [Approval Voting](./examples/exercises/approvalVoting.ts)
- There's a significant difficulty gap here, but there's extensions to the first
  two exercises that can help bridge this gap
- [Sneaky Tic-Tac-Toe](./examples/exercises/sneakyTicTacToe.ts)
- [Asset Swap](./examples/exercises/assetSwap.ts)
- [Poker Hands](./examples/exercises/pokerHands.ts)

[Solutions](https://github.com/privacy-scaling-explorations/summon/tree/exercise-solutions/examples/exerciseSolutions).

## `summon` APIs

In Summon you have access to a special global called `summon`. If you're running
TypeScript, you can copy `summon.d.ts` into your project to get accurate type
information.

## External Bristol Circuits

If you want to use a special boolean function which is available in
[brisol format](https://nigelsmart.github.io/MPC-Circuits/), then you can
convert it to Summon like this:

```sh
cargo run --bin bristol_to_summon -- -i sha256.txt -o sha256.ts
```

```ts
/** generated using bristol_to_summon */
export default function sha256(
  input0: boolean[],
  input1: boolean[],
): boolean[] {
  if (input0.length !== 512) throw new Error("input0 length");
  if (input1.length !== 256) throw new Error("input1 length");

  let w = [...input0, ...input1];
  while (w.length < 1759) w.push(false);

  for (const [dst, op, in0, in1] of gates as any) {
    switch (op) {
      case "INV":
        w[dst] = !w[in0];
        break;
      case "XOR":
        w[dst] = w[in0] !== w[in1];
        break;
      case "AND":
        w[dst] = w[in0] && w[in1];
        break;
    }
  }

  return w.slice(1503, 1503 + 256);
}

const gates = [
  [768, "AND", 416, 576],
  [769, "XOR", 416, 576],
  [770, "XOR", 417, 577],
  [771, "XOR", 418, 578],
  [772, "XOR", 419, 579],
  // etc
];
```

This particular circuit is available as `sha256/mod.ts` in
[summon-lib](https://github.com/privacy-scaling-explorations/summon-lib). It
also comes wrapped in a function which correctly implements chunking and padding
so you can easily calculate sha256 over any number of bits.

## Why "Summon"

The circuits generated by Summon are intended for MPC. When performing MPC, you
use cryptography to collaboratively compute the output of a function without
anyone seeing each other's inputs or any of the intermediary calculations. It's
like _summoning_ the result with magic.
