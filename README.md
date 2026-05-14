# observing components

Vite plugin to make React components observers of reactive state.

## Motivation

By default React's state management is based on props passing. In short, when using React you will have a lot of components with a lot of props. And because of React relying on immutable state, those props will very often change. That means you do not want to wrap every component in `memo`, because you will more often than not compare props that typically change, just adding overhead to the reconciliation.

But when you build a reactive application where you have root state that is mutable, for example with [Mobx](https://mobx.js.org/README.html), there is much less props passing and the props passed are often mutable objects that will never change, but are rather observed in the nested component. In this world it makes a lot of sense to make all components observers of reactive state and memoed for optimal performance.

The great thing about this is how the mental model of rendering changes. Components will only render when the state they actually access changes. There will be no waterfall reconciliation happening and you optimise your rendering by simply splitting up your components. No memoizing hooks or anything.

**Other benefits:**

- You can now export functions as normal and they show up with the correct name in React Devtools
- When exporting with `export const Comp = observer()` VSCode will read that as two definitions of the component, affecting "jump to definition". Now there is only one definition for every component

## How to

```sh
npm install vite-plugin-observing-components
```

```ts
import { observingComponents } from "vite-plugin-observing-components";

export default defineConfig({
  plugins: [
    observingComponents({
      importPath: "mobx-react-lite", // module that exports observer
      importName: "observer",        // default
      exclude: ["src/ui-components/**"], // optional
    }),
  ],
});
```

Works with any Vite version and any underlying transformer (SWC or Babel) — no extra configuration needed.

> **Note:** The `babel-plugin-observing-components` and `swc-plugin-observing-components` packages are deprecated. Use this plugin instead.
