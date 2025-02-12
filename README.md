# observing components

Babel and SWC plugin to make components observers

## Motivation

By default Reacts state management is based on props passing. In short, when using React you will have a lot of components with a lot of props. And because of React relying on immutable state, those props will very often change. That means you do not want to wrap every component in `memo`, because you will more often than not compare props that typically changes, just adding overhead to the reconciliation.

But when you build a reactive application where you have root state that is mutable, for example with [Mobx]() or [Bonsify](), there is much less props passing and the props passed are often mutable objects that will never change, but is rather observed in the nested component. In this world it makes a lot of sense to make all components observers of reactive state and memoed for optimal performance.

The great thing about this is how the mental model of rendering changes. Components will only render when the state they access actually change. There will be no waterfall reconciliation happening and you optimise your rendering by simply splitting up your components. No memoizing hooks or anything.

## How to

**Babel**

```sh
npm install babel-plugin-observing-components
```

```ts
import observingComponents from "babel-plugin-observing-components";

const babelConfig = {
  plugins: [
    observingComponents({
      importName: "observer", // Default
      importPath: "bonsify",
    }),
  ],
};
```

**SWC**

```sh
npm install babel-plugin-observing-components
```

```json
{
  "plugins": [
    [
      "swc-plugin-observing-components",
      {
        "import_path": "bonsify",
        "import_name": "observer" // default
      }
    ]
  ]
}
```
