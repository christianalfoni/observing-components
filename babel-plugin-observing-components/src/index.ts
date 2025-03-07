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
  isObjectProperty,
  isObjectExpression,
  ObjectProperty,
  isMemberExpression,
} from "@babel/types";

// Helper function to check if identifier starts with uppercase letter
function startsWithUppercase(name: string): boolean {
  return /^[A-Z]/.test(name);
}

// Helper function to check if a function is a component (has JSX and starts with uppercase)
function isComponentFunction(path: NodePath<Node>): boolean {
  // Check for JSX
  let hasJsx = false;
  path.traverse({
    JSX() {
      hasJsx = true;
      // Don't call path.stop() here as it stops traversal for the entire parent path
    },
  });

  if (!hasJsx) return false;

  // Check if the identifier starts with uppercase
  let functionName: string | null = null;

  if (isFunctionDeclaration(path.node) && path.node.id) {
    // Standalone function declaration
    functionName = path.node.id.name;
  } else {
    // For function expressions, we need to look up further in the AST
    let currentPath: NodePath = path;
    let found = false;

    // Look up through ancestors to find variable declarator, assignment, or object property
    while (currentPath.parentPath && !found) {
      currentPath = currentPath.parentPath;

      // Direct variable declaration: const Component = () => {}
      if (
        isVariableDeclarator(currentPath.node) &&
        isIdentifier(currentPath.node.id)
      ) {
        functionName = currentPath.node.id.name;
        found = true;
      }
      // Skip object properties - we'll handle these separately
      else if (isObjectProperty(currentPath.node)) {
        // Don't process object properties here
        return false;
      }
      // Handle function wrapped in another function: const Component = foo(() => {})
      else if (isCallExpression(currentPath.node)) {
        // Keep looking up
        continue;
      }
      // If we reach a statement level without finding an identifier, stop looking
      else if (currentPath.isStatement()) {
        break;
      }
    }
  }

  return functionName !== null && startsWithUppercase(functionName);
}

// Helper function to check if property name starts with uppercase
function isUppercaseProperty(prop: ObjectProperty): boolean {
  // Regular property: { Foo: () => jsx }
  if (!prop.computed && isIdentifier(prop.key)) {
    return startsWithUppercase(prop.key.name);
  }
  // Computed property with MemberExpression: { [PageAction.List]: () => jsx }
  else if (prop.computed && isMemberExpression(prop.key)) {
    const object = prop.key.object;
    if (isIdentifier(object)) {
      return startsWithUppercase(object.name);
    }
  }
  // Computed property with Identifier: { [Foo]: () => jsx }
  else if (prop.computed && isIdentifier(prop.key)) {
    return startsWithUppercase(prop.key.name);
  }
  // For any other cases, just check the string representation if possible
  else if (prop.key && prop.key.loc && prop.key.loc.identifierName) {
    return startsWithUppercase(prop.key.loc.identifierName);
  }
  return false;
}

// Helper function to check if a property value has JSX
function propertyHasJSX(propertyPath: NodePath<ObjectProperty>): boolean {
  let hasJsx = false;

  propertyPath.get("value").traverse({
    JSX() {
      hasJsx = true;
      // Don't call propertyPath.stop() here
    },
  });

  return hasJsx;
}

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
      let transformedJSX = false; // renamed flag

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

      // Handle object properties with JSX functions
      path.traverse({
        ObjectProperty(propertyPath) {
          const prop = propertyPath.node;

          // Skip if not a function value
          if (
            !isFunctionExpression(prop.value) &&
            !isArrowFunctionExpression(prop.value)
          ) {
            return;
          }

          // Check if property key starts with uppercase and has JSX
          if (isUppercaseProperty(prop) && propertyHasJSX(propertyPath)) {
            // Check if already wrapped with observer
            if (
              isCallExpression(prop.value) &&
              // @ts-ignore
              isIdentifier(prop.value.callee) &&
              // @ts-ignore
              prop.value.callee.name === IMPORT_NAME
            ) {
              return;
            }

            const observerCall = callExpression(identifier(IMPORT_NAME), [
              prop.value,
            ]);
            propertyPath.get("value").replaceWith(observerCall);
            transformedJSX = true;
          }
        },
      });

      // Wrap any function returning JSX with observer
      path.traverse({
        FunctionDeclaration(functionPath) {
          if (isComponentFunction(functionPath)) {
            const hasWrapped = wrapFunctionWithObserver(
              functionPath,
              IMPORT_NAME
            );

            transformedJSX = transformedJSX || hasWrapped;

            functionPath.skip();
          }
        },
        FunctionExpression(functionPath) {
          // Skip if parent is ObjectProperty - we handle that separately
          if (isObjectProperty(functionPath.parent)) {
            return;
          }

          if (isComponentFunction(functionPath)) {
            const hasWrapped = wrapFunctionWithObserver(
              functionPath,
              IMPORT_NAME
            );
            transformedJSX = transformedJSX || hasWrapped;

            functionPath.skip();
          }
        },
        ArrowFunctionExpression(functionPath) {
          // Skip if parent is ObjectProperty - we handle that separately
          if (isObjectProperty(functionPath.parent)) {
            return;
          }

          if (isComponentFunction(functionPath)) {
            const hasWrapped = wrapFunctionWithObserver(
              functionPath,
              IMPORT_NAME
            );

            transformedJSX = transformedJSX || hasWrapped;

            functionPath.skip();
          }
        },
      });

      // Only add import if functions with JSX were transformed
      if (!hasObserverImport && transformedJSX) {
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
  const parentNode = functionPath.parent;
  const parentPath = functionPath.parentPath;

  // Check if the function is already wrapped with observer
  if (
    isCallExpression(parentNode) &&
    isIdentifier(parentNode.callee) &&
    parentNode.callee.name === observerName
  ) {
    // If we're in a variable declaration, leave it as is
    if (parentPath?.parent && isVariableDeclarator(parentPath.parent)) {
      return false;
    }
    // Otherwise, wrap the entire observer call
    const observerCall = callExpression(identifier(observerName), [parentNode]);
    if (parentPath) {
      parentPath.replaceWith(observerCall);
      return true;
    }
    return false;
  }

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
      return true;
    }

    if (id) {
      const _variableDeclarator = variableDeclarator(id, observerCall);
      const _variableDeclaration = variableDeclaration("const", [
        _variableDeclarator,
      ]);

      // Replace the function declaration with variable declaration
      functionPath.replaceWith(_variableDeclaration);

      return true;
    }
  }

  // For VariableDeclarators (e.g., const Func = () => {})
  if (
    isVariableDeclarator(funcNode) &&
    (isFunctionExpression(funcNode.init) ||
      isArrowFunctionExpression(funcNode.init))
  ) {
    const init = funcNode.init;
    const observerCall = callExpression(identifier(observerName), [init]);
    // @ts-ignore
    functionPath.get("init").replaceWith(observerCall);
    return true;
  }

  // For other cases where it's not already wrapped
  const observerCall = callExpression(identifier(observerName), [
    // @ts-ignore
    funcNode,
  ]);

  functionPath.replaceWith(observerCall);
  return true;
}

export default function createPlugin(options: {
  importPath: string;
  importName?: string;
}): [typeof transform, Record<string, any>] {
  return [
    transform,
    {
      importPath: options.importPath,
      importName: options.importName || "observer",
    },
  ];
}
