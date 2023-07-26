//! test_output(false)
// Should be: true

// When functions (and therefore classes) have the same source (and references), they should compare
// equal due to the content hash system. This isn't implemented yet - the content hash is just faked
// using its location in the bytecode, which means you only get equal comparison when the functions
// are referentially equal (inconsistent with value semantics).

export default () => {
  const a = new classes[0](1, 2);
  const b = new classes[1](1, 2);

  return a === b;
};

const classes = [
  class Point {
    constructor(public x: number, public y: number) {}

    lenSq() {
      return this.x ** 2 + this.y ** 2;
    }
  },
  class Point {
    constructor(public x: number, public y: number) {}

    lenSq() {
      return this.x ** 2 + this.y ** 2;
    }
  },
];
