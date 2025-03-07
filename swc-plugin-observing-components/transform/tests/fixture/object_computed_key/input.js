const FOO = "Foo";
const bar = "bar";

const components = {
  // This should be transformed (uppercase computed key)
  [FOO]: () => <div />,

  // This should NOT be transformed (lowercase computed key)
  [bar]: () => <div />,
};

export default components;
