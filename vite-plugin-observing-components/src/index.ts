import { transformAsync } from "@babel/core";
import createPlugin from "babel-plugin-observing-components";
import type { Plugin } from "vite";

export interface ObservingComponentsOptions {
  /**
   * The module path that exports `observer`.
   * @example "reactx"
   */
  importPath: string;
  /**
   * The name of the observer export from importPath.
   * @default "observer"
   */
  importName?: string;
  /**
   * Glob patterns (relative to cwd) to exclude from transformation.
   * @example ["src/generated/**"]
   */
  exclude?: string[];
}

export function observingComponents(
  options: ObservingComponentsOptions
): Plugin {
  return {
    name: "vite-plugin-observing-components",
    enforce: "pre",
    async transform(code, id) {
      // Only process JS/TS/JSX/TSX files
      if (!/\.[jt]sx?$/.test(id)) return null;
      // Skip node_modules
      if (id.includes("node_modules")) return null;

      const result = await transformAsync(code, {
        filename: id,
        plugins: [createPlugin(options)],
        sourceMaps: true,
        // Don't read any project babel config — only our plugin runs here
        configFile: false,
        babelrc: false,
        parserOpts: {
          plugins: ["jsx", "typescript"],
        },
      });

      if (!result || result.code == null) return null;

      return {
        code: result.code,
        map: result.map ?? null,
      };
    },
  };
}
