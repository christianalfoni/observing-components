import { describe, test, expect } from "vitest";
import { join } from "path";
import { observingComponents } from "./index";
import type { Plugin } from "vite";

const BASE_OPTIONS = { importPath: "reactx" };

// Vite always provides absolute paths rooted at cwd — mirror that in tests.
const cwd = process.cwd();
const src = (file: string) => join(cwd, "src", file);

async function runTransform(
  code: string,
  id = src("Component.tsx")
): Promise<string | null> {
  const plugin = observingComponents(BASE_OPTIONS) as Plugin & {
    transform: (code: string, id: string) => Promise<{ code: string } | null>;
  };
  const result = await plugin.transform(code, id);
  return result?.code ?? null;
}

// ── Plugin metadata ──────────────────────────────────────────────────────────

describe("plugin metadata", () => {
  test("has the correct name", () => {
    const plugin = observingComponents(BASE_OPTIONS) as Plugin;
    expect(plugin.name).toBe("vite-plugin-observing-components");
  });

  test('enforces "pre" so it runs before the JSX transform', () => {
    const plugin = observingComponents(BASE_OPTIONS) as Plugin;
    expect(plugin.enforce).toBe("pre");
  });
});

// ── File filtering ───────────────────────────────────────────────────────────

describe("file filtering", () => {
  test("skips node_modules", async () => {
    const code = `export function Counter() { return <div /> }`;
    const result = await runTransform(
      code,
      join(cwd, "node_modules", "some-lib", "Component.tsx")
    );
    expect(result).toBeNull();
  });

  test("skips non-JS/TS files", async () => {
    const result = await runTransform(
      `body { color: red }`,
      src("styles.css")
    );
    expect(result).toBeNull();
  });

  test("processes .tsx files", async () => {
    const result = await runTransform(
      `export function Counter() { return <div /> }`,
      src("Counter.tsx")
    );
    expect(result).not.toBeNull();
  });

  test("processes .ts files", async () => {
    // No JSX — nothing to wrap, but should not throw
    const result = await runTransform(
      `export function helper() { return 42; }`,
      src("helper.ts")
    );
    expect(result).not.toBeNull();
  });

  test("processes .jsx files", async () => {
    const result = await runTransform(
      `export function Counter() { return <div /> }`,
      src("Counter.jsx")
    );
    expect(result).not.toBeNull();
  });

  test("processes .js files", async () => {
    const result = await runTransform(
      `export function helper() { return 42; }`,
      src("helper.js")
    );
    expect(result).not.toBeNull();
  });
});

// ── Wrapping behaviour ───────────────────────────────────────────────────────

describe("component wrapping", () => {
  test("wraps an exported function component", async () => {
    const result = await runTransform(
      `export function Counter() { return <div /> }`
    );
    expect(result).toContain('import { observer } from "reactx"');
    expect(result).toContain("observer(function Counter()");
  });

  test("wraps an arrow function component", async () => {
    const result = await runTransform(
      `export const Counter = () => <div />`
    );
    expect(result).toContain('import { observer } from "reactx"');
    expect(result).toContain("observer(");
  });

  test("does not wrap a function without JSX", async () => {
    const result = await runTransform(
      `export function helper() { return 42; }`
    );
    expect(result).not.toContain("observer");
  });

  test("does not wrap a lowercase function even with JSX", async () => {
    const result = await runTransform(
      `export function counter() { return <div /> }`
    );
    expect(result).not.toContain("observer");
  });

  test("does not double-wrap an already-wrapped component", async () => {
    const result = await runTransform(
      `import { observer } from "reactx";
const Counter = observer(() => <div />);`
    );
    // observer should appear in the import and once wrapping — not twice
    const observerCallCount = (result?.match(/observer\(/g) ?? []).length;
    expect(observerCallCount).toBe(1);
  });

  test("does not wrap components inside object literals", async () => {
    const result = await runTransform(
      `const components = { Foo: () => <div /> }`
    );
    expect(result).not.toContain("observer");
  });

  test("respects custom importName option", async () => {
    const plugin = observingComponents({
      importPath: "my-lib",
      importName: "reactive",
    }) as Plugin & {
      transform: (code: string, id: string) => Promise<{ code: string } | null>;
    };
    const result = await plugin.transform(
      `export function Counter() { return <div /> }`,
      src("Counter.tsx")
    );
    expect(result?.code).toContain('import { reactive } from "my-lib"');
    expect(result?.code).toContain("reactive(function Counter()");
  });
});

// ── Exclude patterns ─────────────────────────────────────────────────────────

describe("exclude option", () => {
  test("skips files matching an exclude glob", async () => {
    const plugin = observingComponents({
      ...BASE_OPTIONS,
      exclude: ["src/generated/**"],
    }) as Plugin & {
      transform: (code: string, id: string) => Promise<{ code: string } | null>;
    };
    const result = await plugin.transform(
      `export function Counter() { return <div /> }`,
      src("generated/Counter.tsx")
    );
    expect(result?.code).not.toContain("observer");
  });

  test("still transforms files not matching the exclude glob", async () => {
    const plugin = observingComponents({
      ...BASE_OPTIONS,
      exclude: ["src/generated/**"],
    }) as Plugin & {
      transform: (code: string, id: string) => Promise<{ code: string } | null>;
    };
    const result = await plugin.transform(
      `export function Counter() { return <div /> }`,
      src("ui/Counter.tsx")
    );
    expect(result?.code).toContain("observer");
  });
});

// ── Source maps ──────────────────────────────────────────────────────────────

describe("source maps", () => {
  test("returns a source map alongside the transformed code", async () => {
    const plugin = observingComponents(BASE_OPTIONS) as Plugin & {
      transform: (
        code: string,
        id: string
      ) => Promise<{ code: string; map: unknown } | null>;
    };
    const result = await plugin.transform(
      `export function Counter() { return <div /> }`,
      src("Counter.tsx")
    );
    expect(result?.map).not.toBeNull();
  });
});
