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
});
