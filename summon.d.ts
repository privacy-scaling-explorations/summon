declare const summon: {
  /**
   * Tests whether the provided `value` is a signal.
   *
   * Signals are used to generate circuits during partial evaluation by representing computations
   * over unknown values.
   *
   *     const x = summon.inputPublic('x', summon.number());
   *     const y = 17;
   *
   *     summon.isSignal(x); // true
   *     summon.isSignal(y); // false
   *     summon.isSignal(x + y); // true
   */
  isSignal(value: unknown): boolean;

  /** Produces a runtime value that models the type `number`. */
  number(): Summon.Type<number>;

  /** Produces a runtime value that models the type `boolean`. */
  bool(): Summon.Type<boolean>;
};

declare namespace Summon {
  export type IO = {
    /** Accept an input from a specific party. */
    input<T>(from: string, name: string, type: Type<T>): T;

    /** Accept a compile-time input (all parties must provide the same value). */
    inputPublic<T>(name: string, type: Type<T>): T;

    /** Provide an output visible only to a specific party or parties. */
    output<T>(to: string | string[], name: string, value: T): void;

    /** Provide an output visible to all parties. */
    outputPublic<T>(name: string, value: T): void;

    /**
     * Explicitly add an MPC party.
     *
     * Usually parties are inferred from inputs and outputs, but you can also have parties with no
     * inputs and only receive public outputs. Use this method to declare them so they will be
     * included in mpcSettings.
     */
    addParty(name: string): void;
  };

  export type Type<T> = {
    about: 'summon runtime type',
    json: unknown,

    // This is just here to help keep TypeScript aware of T. This field is never supposed to
    // actually be present.
    _typeCheck?: (x: T) => T,
  };

  export type TypeOf<T> = T extends Type<infer U> ? U : never;
}
