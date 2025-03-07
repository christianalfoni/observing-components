const components = {
  // This should NOT be transformed (uppercase key)
  Foo: () => <div />,

  // This should NOT be transformed (lowercase key)
  bar: () => <div />,
};

export default components;
