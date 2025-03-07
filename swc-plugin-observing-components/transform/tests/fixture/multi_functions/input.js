// This should be transformed (uppercase)
const Foo = () => <div />;

// This should NOT be transformed (lowercase)
const bar = () => <div />;

// This should be transformed (uppercase)
const Baz = () => <div />;

export { Foo, bar, Baz };
