import { describe, test, expect } from "vitest";
import { transformSync } from "@babel/core";
import transform from "./";

function runTransform(input: string) {
  const result = transformSync(input, {
    plugins: [
      "@babel/plugin-syntax-jsx",
      transform({
        importPath: "bonsify",
      }),
    ],
  });

  return result?.code;
}

describe("transform", () => {
  test("Does not transforms function without jsx", () => {
    expect(
      runTransform(`const Counter = () => {
  return
}`)
    ).toBe(`const Counter = () => {
  return;
};`);
  });
  test("Transforms variable arrow function with jsx", () => {
    expect(
      runTransform(`const Counter = () => {
    return <h1>Hello</h1>
}`)
    ).toBe(`import { observer } from "bonsify";
const Counter = observer(() => {
  return <h1>Hello</h1>;
});`);
  });

  test("Transforms variable arrow function with existing higher order function", () => {
    expect(
      runTransform(`const Counter = foo(() => {
    return <h1>Hello</h1>
})`)
    ).toBe(`import { observer } from "bonsify";
const Counter = foo(observer(() => {
  return <h1>Hello</h1>;
}));`);
  });

  test("Transforms function with jsx", () => {
    expect(
      runTransform(`const Counter = function () {
    return <h1>Hello</h1>
}`)
    ).toBe(`import { observer } from "bonsify";
const Counter = observer(function () {
  return <h1>Hello</h1>;
});`);
  });

  test("Transforms standalone function with jsx", () => {
    expect(
      runTransform(`function Counter6 () {
    return <h1>Hello</h1>
}`)
    ).toBe(`import { observer } from "bonsify";
const Counter6 = observer(function Counter6() {
  return <h1>Hello</h1>;
});`);
  });

  test("Should not transform if already observer", () => {
    expect(
      runTransform(`const Counter6 = observer(() => {
    return <h1>Hello</h1>
})`)
    ).toBe(`const Counter6 = observer(() => {
  return <h1>Hello</h1>;
});`);
  });

  test("Should wrap existing wrappers", () => {
    expect(
      runTransform(`const Counter6 = foo(() => {
    return <h1>Hello</h1>
})`)
    ).toBe(`import { observer } from "bonsify";
const Counter6 = foo(observer(() => {
  return <h1>Hello</h1>;
}));`);
  });

  test("Should not transform function with lowercase name even with jsx", () => {
    expect(
      runTransform(`const counter = () => {
    return <h1>Hello</h1>
}`)
    ).toBe(`const counter = () => {
  return <h1>Hello</h1>;
};`);
  });

  test("Should transform function with uppercase name and jsx", () => {
    expect(
      runTransform(`const Counter = () => {
    return <h1>Hello</h1>
}`)
    ).toBe(`import { observer } from "bonsify";
const Counter = observer(() => {
  return <h1>Hello</h1>;
});`);
  });

  test("Should not transform standalone function with lowercase name and jsx", () => {
    expect(
      runTransform(`function counter() {
    return <h1>Hello</h1>
}`)
    ).toBe(`function counter() {
  return <h1>Hello</h1>;
}`);
  });

  test("Should transform other functions when ignoring a function that should not be transformed", () => {
    expect(
      runTransform(`function counter() {
    return <h1>Hello</h1>
}
function Counter() {
  return <h1>Hello</h1>
}`)
    ).toBe(`import { observer } from "bonsify";
function counter() {
  return <h1>Hello</h1>;
}
const Counter = observer(function Counter() {
  return <h1>Hello</h1>;
});`);
  });

  test("Should transform standalone function with uppercase name and jsx", () => {
    expect(
      runTransform(`function Counter() {
    return <h1>Hello</h1>
}`)
    ).toBe(`import { observer } from "bonsify";
const Counter = observer(function Counter() {
  return <h1>Hello</h1>;
});`);
  });

  test("Should NOT transform components inside object literals", () => {
    expect(
      runTransform(`const components = {
    Foo: () => <div />
}`)
    ).toBe(`const components = {
  Foo: () => <div />
};`);
  });
});
