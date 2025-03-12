// //! test_output(true)
// FIXME: This passes in ValueScript. Somehow a regression has occurred.

export default function () {
  return functions[0]() === functions[1]();
}

const functions = [
  function foo() {
    function content() {
      return "foo";
    }

    return function test() {
      return content();
    };
  },
  function foo() {
    function content() {
      return "foo";
    }

    return function test() {
      return content();
    };
  },
];
