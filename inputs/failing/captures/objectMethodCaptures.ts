//! test_output("bar:undefined")

// This is wrong. It should be:
// //! test_output("bar:bar")

export default function main() {
  const bar = echo("bar");

  const foo = {
    bar() {
      return `bar:${bar}`;
    },
  };

  return foo.bar();
}

function echo<T>(x: T) {
  return x;
}
