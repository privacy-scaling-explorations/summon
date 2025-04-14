declare const summon: {
  isSignal(value: unknown): boolean;
  number(): Summon.Type<number>;
};

declare namespace Summon {
  export type IO = {
    input<T>(from: string, id: string, type: Type<T>): T;
    output<T>(to: string, id: string, value: T): void;

    publicInput<T>(id: string, type: Type<T>): T;
    publicOutput<T>(id: string, value: T): void;
  };

  type Type<T> = {
    about: 'summon runtime type',
    json: unknown,

    // This is just here to help keep TypeScript aware of T. This field is never supposed to
    // actually be present.
    _typeCheck?: (x: T) => T,
  };
}
