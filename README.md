# observing components

Babel and SWC plugin to make components observers

## Motivation

By default Reacts state management is based on props passing. In short, when using React you will have a lot of components with a lot of props. And because of React relying on immutable state, those props will very often change. That means you do not want to wrap every component in `memo`, because you will more often than not compare props that typically changes, just adding overhead to the reconciliation.

But when you build a reactive application where you have root state that is mutable, for example with [Mobx](https://mobx.js.org/README.html), there is much less props passing and the props passed are often mutable objects that will never change, but is rather observed in the nested component. In this world it makes a lot of sense to make all components observers of reactive state and memoed for optimal performance.

The great thing about this is how the mental model of rendering changes. Components will only render when the state they actually access changes. There will be no waterfall reconciliation happening and you optimise your rendering by simply splitting up your components. No memoizing hooks or anything.

**Other benefits:**

- You can now export functions as normal and they show up with the correct name in React Devtools
- When exporting with `export const Comp = observer()` VSCode will read that as two definitions of the component, affecting "jump to definition". Now there is only one definition for every component

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
      exclude: ["src/ui-components/**"], // optional
    }),
  ],
};
```

**SWC**

```sh
npm install swc-plugin-observing-components
```

```json
{
  "plugins": [
    [
      "swc-plugin-observing-components",
      {
        "import_name": "observer", // default
        "import_path": "bonsify",
        "exclude": ["src/ui-components/**"] // optional
      }
    ]
  ]
}
```
