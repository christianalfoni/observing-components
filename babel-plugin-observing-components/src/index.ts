import { PluginObj } from "@babel/core";
import { NodePath } from "@babel/traverse";
// Explicit imports to avoid tslib dependency
import {
  isIdentifier,
  identifier,
  isImportSpecifier,
  importSpecifier,
  isFunctionDeclaration,
  isFunctionExpression,
  isArrowFunctionExpression,
  stringLiteral,
  importDeclaration,
  functionExpression,
  callExpression,
  Node,
  isExportDefaultDeclaration,
  variableDeclarator,
  variableDeclaration,
  isVariableDeclarator,
  isCallExpression,
} from "@babel/types";

export const transform: PluginObj = {
  name: "wrap-with-observer",
  visitor: {
    Program(path, state) {
      // @ts-ignore
      const filename = state.filename || state.opts.filename;
      // @ts-ignore
      const IMPORT_PATH = state.opts.importPath as string;
      // @ts-ignore
      const IMPORT_NAME: string = state.opts.importName as string;

      if (filename && !shouldProcessFile(filename)) {
        return;
      }

      let hasObserverImport = false;

      path.traverse({
        ImportDeclaration(importPath) {
          if (importPath.node.source.value === IMPORT_PATH) {
            const hasObserverSpecifier = importPath.node.specifiers.some(
              (specifier) =>
                isImportSpecifier(specifier) &&
                isIdentifier(specifier.imported) &&
                specifier.imported.name === IMPORT_NAME
            );
            if (!hasObserverSpecifier) {
              importPath.node.specifiers.push(
                importSpecifier(
                  identifier(IMPORT_NAME),
                  identifier(IMPORT_NAME)
                )
              );
            }
            hasObserverImport = true;
          }
        },
      });

      // New: wrap any function returning JSX with observer
      path.traverse({
        FunctionDeclaration(functionPath) {
          let hasJsx = false;
          functionPath.traverse({
            JSX() {
              hasJsx = true;
            },
          });
          if (hasJsx) {
            wrapFunctionWithObserver(functionPath, IMPORT_NAME);
            functionPath.skip();
          }
        },
        FunctionExpression(functionPath) {
          let hasJsx = false;
          functionPath.traverse({
            JSX() {
              hasJsx = true;
            },
          });
          if (hasJsx) {
            wrapFunctionWithObserver(functionPath, IMPORT_NAME);
            functionPath.skip();
          }
        },
        ArrowFunctionExpression(functionPath) {
          let hasJsx = false;
          functionPath.traverse({
            JSX() {
              hasJsx = true;
            },
          });
          if (hasJsx) {
            wrapFunctionWithObserver(functionPath, IMPORT_NAME);
            functionPath.skip();
          }
        },
      });

      if (!hasObserverImport) {
        const _importDeclaration = importDeclaration(
          [importSpecifier(identifier(IMPORT_NAME), identifier(IMPORT_NAME))],
          stringLiteral(IMPORT_PATH)
        );
        path.unshiftContainer("body", _importDeclaration);
      }
    },
  },
};

function shouldProcessFile(filename: string) {
  const isNodeModule = filename.includes("node_modules");
  const isProjectFile = filename.startsWith(process.cwd());
  return !isNodeModule && isProjectFile;
}

function wrapFunctionWithObserver(
  functionPath: NodePath<Node>,
  observerName: string
) {
  const funcNode = functionPath.node;

  if (isFunctionDeclaration(funcNode)) {
    // Convert FunctionDeclaration to VariableDeclaration wrapped with observer
    const id = funcNode.id;
    const funcExpression = functionExpression(
      funcNode.id,
      funcNode.params,
      funcNode.body,
      funcNode.generator,
      funcNode.async
    );

    const observerCall = callExpression(identifier(observerName), [
      funcExpression,
    ]);

    if (isExportDefaultDeclaration(functionPath.parent)) {
      functionPath.replaceWith(observerCall);
    } else if (id) {
      const _variableDeclarator = variableDeclarator(id, observerCall);
      const _variableDeclaration = variableDeclaration("const", [
        _variableDeclarator,
      ]);

      // Replace the function declaration with variable declaration
      functionPath.replaceWith(_variableDeclaration);
    }
  }

  // For VariableDeclarators (e.g., const Func = () => {})
  else if (
    isVariableDeclarator(funcNode) &&
    (isFunctionExpression(funcNode.init) ||
      isArrowFunctionExpression(funcNode.init))
  ) {
    const init = funcNode.init;

    const observerCall = callExpression(identifier(observerName), [init]);

    // @ts-ignore
    functionPath.get("init").replaceWith(observerCall);
  }

  // For other cases
  else if (
    !isCallExpression(functionPath.parent) ||
    // @ts-ignore
    functionPath.parent.callee.name !== observerName
  ) {
    const observerCall = callExpression(identifier(observerName), [
      // @ts-ignore
      funcNode,
    ]);

    functionPath.replaceWith(observerCall);
  }
}

export default function createPlugin({
  importPath,
  importName = "observer",
}: {
  importPath: string;
  importName?: string;
}): [typeof transform, Record<string, any>] {
  return [transform, { importPath, importName }];
}
