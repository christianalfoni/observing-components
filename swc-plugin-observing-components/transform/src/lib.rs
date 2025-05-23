use swc_ecma_ast::*;
use swc_ecma_visit::{fold_pass, noop_fold_type, Fold};
use serde::Deserialize;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub import_name: Option<String>,
    pub import_path: String,
    #[serde(default)]
    pub exclude: Vec<String>,
}

// Helper function to check if a path should be excluded
pub fn should_exclude(file_path: &str, exclude_patterns: &[String]) -> bool {
    if exclude_patterns.is_empty() {
        return false;
    }

    let path = Path::new(file_path);
    
    // Create PathBuf from the full path
    let path_buf = PathBuf::from(file_path);
    
    // Get just the file name for simpler matching
    let file_name = path.file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    
    // Get the components of the path for more precise matching
    let path_components: Vec<&str> = file_path.split(std::path::MAIN_SEPARATOR).collect();
    
    let mut builder = GlobSetBuilder::new();
    
    for pattern in exclude_patterns {
        // Create a glob for each pattern and add to the set
        match Glob::new(pattern) {
            Ok(glob) => { builder.add(glob); },
            Err(_) => { 
                eprintln!("Invalid glob pattern: {}", pattern);
                continue;
            }
        }
    }

    match builder.build() {
        Ok(globset) => {
            // Try to match against the full absolute path
            if globset.is_match(file_path) {
                return true;
            }
            
            // Try to match against just the file name
            if globset.is_match(file_name) {
                return true;
            }
            
            // Check if any pattern matches the path components
            // This helps with relative path patterns like "src/Test.tsx"
            for pattern in exclude_patterns {
                let pattern_components: Vec<&str> = pattern.split('/').collect();
                
                // Try to find the pattern components as a subsequence in the path
                if path_components_match(&path_components, &pattern_components) {
                    return true;
                }
            }
            
            // Also try to match against path relative from project root
            // This handles cases where exclude pattern is like "src/Test.tsx"
            for i in 0..path_components.len() {
                let potential_project_root = path_components[0..i].join(&std::path::MAIN_SEPARATOR.to_string());
                let potential_relative_path = path_components[i..].join(&std::path::MAIN_SEPARATOR.to_string());
                
                if globset.is_match(&potential_relative_path) {
                    return true;
                }
            }
            
            false
        },
        Err(_) => {
            eprintln!("Failed to build globset from patterns");
            false
        }
    }
}

// Helper function to check if pattern components appear as a subsequence in path components
fn path_components_match(path_components: &[&str], pattern_components: &[&str]) -> bool {
    if pattern_components.is_empty() {
        return true;
    }
    
    // Look for the pattern components in sequence within the path components
    let mut path_idx = 0;
    let mut pattern_idx = 0;
    
    while path_idx < path_components.len() && pattern_idx < pattern_components.len() {
        if path_components[path_idx].to_lowercase() == pattern_components[pattern_idx].to_lowercase() {
            pattern_idx += 1;
            if pattern_idx == pattern_components.len() {
                return true;
            }
        }
        path_idx += 1;
    }
    
    false
}

fn default_import_name() -> String {
    "observer".to_string()
}

pub fn observer_transform(config: Config) -> impl Pass {
    fold_pass(ObserverTransform {
        has_added_import: false,
        config,
    })
}

struct ObserverTransform {
    has_added_import: bool,
    config: Config,
}

impl ObserverTransform {
    fn get_import_name(&self) -> String {
        self.config.import_name.clone().unwrap_or_else(|| "observer".to_string())
    }
}

// Updated function to check if property key has uppercase first letter
fn is_component_name(name: &str) -> bool {
    if let Some(first_char) = name.chars().next() {
        first_char.is_uppercase()
    } else {
        false
    }
}

// Removed get_computed_property_name helper function as it's no longer needed

fn contains_jsx_in_expr(expr: &Expr) -> bool {
    match expr {
        Expr::JSXElement(_) | Expr::JSXFragment(_) => true,
        Expr::Paren(e) => contains_jsx_in_expr(&e.expr),
        Expr::Fn(f) => contains_jsx_in_function(&f.function),
        Expr::Arrow(arrow) => {
            if let BlockStmtOrExpr::BlockStmt(block) = &*arrow.body {
                contains_jsx_in_block(block)
            } else if let BlockStmtOrExpr::Expr(expr) = &*arrow.body {
                contains_jsx_in_expr(expr)
            } else {
                false
            }
        },
        // Check if Call expressions' arguments contain JSX
        Expr::Call(call_expr) => call_expr.args.iter().any(|arg| contains_jsx_in_expr(&arg.expr)),
        // Removed Object properties JSX checking
        _ => false
    }
}

fn contains_jsx_in_function(function: &Function) -> bool {
    if let Some(body) = &function.body {
        contains_jsx_in_block(body)
    } else {
        false
    }
}

fn contains_jsx_in_block(block: &BlockStmt) -> bool {
    block.stmts.iter().any(|stmt| contains_jsx_in_stmt(stmt))
}

fn contains_jsx_in_stmt(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Decl(Decl::Fn(fn_decl)) => contains_jsx_in_function(&fn_decl.function), // NEW: check function declarations
        Stmt::Return(ret) => {
            if let Some(expr) = &ret.arg {
                contains_jsx_in_expr(expr)
            } else {
                false
            }
        },
        Stmt::Expr(expr) => contains_jsx_in_expr(&expr.expr),
        Stmt::Block(block) => contains_jsx_in_block(block),
        Stmt::Decl(Decl::Var(var_decl)) => var_decl.decls.iter().any(|decl| {
            if let Some(init) = &decl.init {
                contains_jsx_in_expr(init)
            } else {
                false
            }
        }),
        _ => false
    }
}

fn contains_jsx_in_module(module: &Module) -> bool {
    module.body.iter().any(|item| match item {
        ModuleItem::Stmt(stmt) => match stmt {
            // Add explicit check for variable declarations in module statements
            Stmt::Decl(Decl::Var(var_decl)) => var_decl.decls.iter().any(|decl| {
                if let Some(init) = &decl.init {
                    contains_jsx_in_expr(init)
                } else {
                    false
                }
            }),
            _ => contains_jsx_in_stmt(stmt),
        },
        ModuleItem::ModuleDecl(decl) => match decl {
            ModuleDecl::ExportDefaultExpr(export) => contains_jsx_in_expr(&export.expr),
            ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
                Decl::Fn(fn_decl) => contains_jsx_in_function(&fn_decl.function),
                Decl::Var(var_decl) => var_decl.decls.iter().any(|decl| {
                    if let Some(init) = &decl.init {
                        // Check arrow functions in variable declarations
                        contains_jsx_in_expr(init)
                    } else {
                        false
                    }
                }),
                _ => false,
            },
            ModuleDecl::ExportDefaultDecl(export_decl) => {
                if let swc_ecma_ast::DefaultDecl::Fn(f) = &export_decl.decl {
                    contains_jsx_in_function(&f.function)
                } else {
                    false
                }
            }
            _ => false,
        },
    })
}

// NEW: Helper to check if an expression is already wrapped
fn is_already_wrapped(expr: &Expr, observer_name: &str) -> bool {
    if let Expr::Call(call_expr) = expr {
        if let Callee::Expr(boxed) = &call_expr.callee {
            if let Expr::Ident(id) = &**boxed {
                return id.sym.to_string() == observer_name;
            }
        }
    }
    false
}

// NEW: Update helper to check for wrapped functions in variable declarations as well.
fn module_contains_wrapped_function(module: &Module, observer_name: &str) -> bool {
    module.body.iter().any(|item| match item {
        // Check top-level expression statements.
        ModuleItem::Stmt(Stmt::Expr(expr_stmt)) => {
            if let Expr::Call(call_expr) = &*expr_stmt.expr {
                if let Callee::Expr(boxed) = &call_expr.callee {
                    if let Expr::Ident(id) = &**boxed {
                        return id.sym.to_string() == observer_name;
                    }
                }
            }
            false
        },
        // Check variable declarations.
        ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))) => {
            var_decl.decls.iter().any(|decl| {
                if let Some(init) = &decl.init {
                    is_already_wrapped(init, observer_name)
                } else {
                    false
                }
            })
        },
        // Check export declarations that include variable declarations.
        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => match &export_decl.decl {
            Decl::Var(var_decl) => {
                var_decl.decls.iter().any(|decl| {
                    if let Some(init) = &decl.init {
                        is_already_wrapped(init, observer_name)
                    } else {
                        false
                    }
                })
            },
            _ => false,
        },
        _ => false,
    })
}

impl Fold for ObserverTransform {
    noop_fold_type!();

    fn fold_module(&mut self, mut module: Module) -> Module {
        // ...existing code...
        let should_add_import = contains_jsx_in_module(&module);
        let observer_name = self.get_import_name();

        // NEW: Do not add an import if an already wrapped function is identified.
        if module_contains_wrapped_function(&module, &observer_name) {
            self.has_added_import = true;
        }

        if should_add_import && !self.has_added_import {
            // ...existing import logic...
            let mut observer_alias = observer_name.clone(); // default alias
            let found_alias = module.body.iter().filter_map(|item| {
                if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item {
                    for spec in &import_decl.specifiers {
                        if let ImportSpecifier::Named(named) = spec {
                            let imported = if let Some(imported) = &named.imported {
                                match imported {
                                    ModuleExportName::Ident(ident) => ident.sym.to_string(),
                                    ModuleExportName::Str(s) => s.value.to_string(),
                                }
                            } else {
                                named.local.sym.to_string()
                            };
                            if imported == observer_name {
                                return Some(named.local.sym.to_string());
                            }
                        }
                    }
                }
                None
            }).next();

            if let Some(alias) = found_alias {
                observer_alias = alias;
                self.has_added_import = true;
            } else {
                let import_path = self.config.import_path.clone();
                let import = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: Default::default(),
                    specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                        span: Default::default(),
                        local: Ident::new(observer_name.clone().into(), Default::default(), Default::default()),
                        imported: None,
                        is_type_only: false,
                    })],
                    src: Box::new(Str {
                        span: Default::default(),
                        value: import_path.into(),
                        raw: None,
                    }),
                    type_only: false,
                    with: None,
                    phase: ImportPhase::Evaluation,
                }));
                module.body.insert(0, import);
                self.has_added_import = true;
            }
            // ...existing code...
        }

        let transformed_body = module.body.into_iter().map(|item| {
            // ...existing transformation code...
            match item {
                // ...existing code...
                ModuleItem::Stmt(Stmt::Decl(Decl::Fn(fn_decl))) => {
                    if contains_jsx_in_function(&fn_decl.function) && is_component_name(&fn_decl.ident.sym.to_string()) {
                        let ident = fn_decl.ident.clone();
                        let fn_expr = Expr::Fn(FnExpr {
                            ident: Some(ident.clone()),
                            function: fn_decl.function.clone(),
                        });
                        let wrapped_fn_expr = Expr::Call(CallExpr {
                            span: Default::default(),
                            callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                observer_name.clone().into(),
                                Default::default(),
                                Default::default(),
                            )))),
                            args: vec![ExprOrSpread {
                                spread: None,
                                expr: Box::new(fn_expr),
                            }],
                            type_args: None,
                            ctxt: Default::default(),
                        });
                        let var_decl = VarDecl {
                            span: fn_decl.function.span,
                            ctxt: Default::default(),
                            kind: VarDeclKind::Const,
                            declare: false,
                            decls: vec![VarDeclarator {
                                span: fn_decl.function.span,
                                name: Pat::Ident(BindingIdent {
                                    id: ident,
                                    type_ann: None,
                                }),
                                init: Some(Box::new(wrapped_fn_expr)),
                                definite: false,
                            }],
                        };
                        ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(var_decl))))
                    } else {
                        ModuleItem::Stmt(Stmt::Decl(Decl::Fn(fn_decl)))
                    }
                },
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export))
                    if contains_jsx_in_expr(&export.expr) =>
                {
                    let wrapped_expr = Expr::Call(CallExpr {
                        span: Default::default(),
                        callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                            observer_name.clone().into(),
                            Default::default(),
                            Default::default(),
                        )))),
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: export.expr,
                        }],
                        type_args: None,
                        ctxt: Default::default(),
                    });
                    
                    ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                        span: export.span,
                        expr: Box::new(wrapped_expr),
                    }))
                },
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(mut export_decl)) => {
                    match &mut export_decl.decl {
                        Decl::Fn(fn_decl) => {
                            if contains_jsx_in_function(&fn_decl.function) && is_component_name(&fn_decl.ident.sym.to_string()) {
                                let ident = fn_decl.ident.clone();
                                let fn_expr = Expr::Fn(FnExpr {
                                    ident: Some(ident.clone()),
                                    function: fn_decl.function.clone(),
                                });
                                let wrapped_fn_expr = Expr::Call(CallExpr {
                                    span: Default::default(),
                                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                        observer_name.clone().into(),
                                        Default::default(),
                                        Default::default(),
                                    )))),
                                    args: vec![ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(fn_expr),
                                    }],
                                    type_args: None,
                                    ctxt: Default::default(),
                                });
                                let var_decl = VarDecl {
                                    span: fn_decl.function.span,
                                    ctxt: Default::default(),
                                    kind: VarDeclKind::Const,
                                    declare: false,
                                    decls: vec![VarDeclarator {
                                        span: fn_decl.function.span,
                                        name: Pat::Ident(BindingIdent {
                                            id: ident,
                                            type_ann: None,
                                        }),
                                        init: Some(Box::new(wrapped_fn_expr)),
                                        definite: false,
                                    }],
                                };
                                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                                    span: export_decl.span,
                                    decl: Decl::Var(Box::new(var_decl)),
                                }))
                            } else {
                                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl))
                            }
                        },
                        Decl::Var(var_decl) => {
                            for decl in var_decl.decls.iter_mut() {
                                if let Some(init) = &mut decl.init {
                                    if is_already_wrapped(&*init, &observer_name) {
                                        continue;
                                    }
                                    
                                    // Get variable name for component detection
                                    let var_name = match &decl.name {
                                        Pat::Ident(binding_ident) => Some(binding_ident.id.sym.to_string()),
                                        _ => None
                                    };
                                    
                                    // Check if variable name starts with uppercase (component name)
                                    let is_component = var_name.as_ref()
                                        .map(|name| is_component_name(name))
                                        .unwrap_or(false);
                                    
                                    if is_component && contains_jsx_in_expr(&*init) {
                                        // Handle both direct function expressions and wrapped functions
                                        match &**init {
                                            Expr::Arrow(_) | Expr::Fn(_) => {
                                                let wrapped = Expr::Call(CallExpr {
                                                    span: Default::default(),
                                                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                                        observer_name.clone().into(),
                                                        Default::default(),
                                                        Default::default(),
                                                    )))),
                                                    args: vec![ExprOrSpread {
                                                        spread: None,
                                                        expr: init.clone(),
                                                    }],
                                                    type_args: None,
                                                    ctxt: Default::default(),
                                                });
                                                *init = Box::new(wrapped);
                                            },
                                            // Handle cases like const Home = someWrapper(() => <div />)
                                            Expr::Call(call_expr) => {
                                                let has_jsx_arg = call_expr.args.iter().any(|arg| {
                                                    match &*arg.expr {
                                                        Expr::Arrow(_) | Expr::Fn(_) => contains_jsx_in_expr(&arg.expr),
                                                        _ => false
                                                    }
                                                });
                                                
                                                if has_jsx_arg {
                                                    let wrapped = Expr::Call(CallExpr {
                                                        span: Default::default(),
                                                        callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                                            observer_name.clone().into(),
                                                            Default::default(),
                                                            Default::default(),
                                                        )))),
                                                        args: vec![ExprOrSpread {
                                                            spread: None,
                                                            expr: init.clone(),
                                                        }],
                                                        type_args: None,
                                                        ctxt: Default::default(),
                                                    });
                                                    *init = Box::new(wrapped);
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl))
                        },
                        _ => ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl))
                    }
                },
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl)) => {
                    if let swc_ecma_ast::DefaultDecl::Fn(ref f) = export_decl.decl {
                        if contains_jsx_in_function(&f.function) {
                            let wrapped_expr = Expr::Call(CallExpr {
                                span: Default::default(),
                                callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                    observer_name.clone().into(),
                                    Default::default(),
                                    Default::default(),
                                )))),
                                args: vec![ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Fn(f.clone())),
                                }],
                                type_args: None,
                                ctxt: Default::default(),
                            });
                            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                                span: export_decl.span,
                                expr: Box::new(wrapped_expr),
                            }))
                        } else {
                            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl))
                        }
                    } else {
                        ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl))
                    }
                },
                // Fix non-exported variable declarations
                ModuleItem::Stmt(Stmt::Decl(Decl::Var(mut var_decl))) => {
                    for decl in var_decl.decls.iter_mut() {
                        if let Some(init) = &mut decl.init {
                            if is_already_wrapped(&*init, &observer_name) {
                                continue;
                            }
                            
                            // Get variable name for component detection
                            let var_name = match &decl.name {
                                Pat::Ident(binding_ident) => Some(binding_ident.id.sym.to_string()),
                                _ => None
                            };
                            
                            // Check if variable name starts with uppercase (component name)
                            let is_component = var_name.as_ref()
                                .map(|name| is_component_name(name))
                                .unwrap_or(false);
                            
                            if is_component && contains_jsx_in_expr(&*init) {
                                // Handle both direct function expressions and wrapped functions
                                match &**init {
                                    Expr::Arrow(_) | Expr::Fn(_) => {
                                        let wrapped = Expr::Call(CallExpr {
                                            span: Default::default(),
                                            callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                                observer_name.clone().into(),
                                                Default::default(),
                                                Default::default(),
                                            )))),
                                            args: vec![ExprOrSpread {
                                                spread: None,
                                                expr: init.clone(),
                                            }],
                                            type_args: None,
                                            ctxt: Default::default(),
                                        });
                                        *init = Box::new(wrapped);
                                    },
                                    // Handle cases like const Home = someWrapper(() => <div />)
                                    Expr::Call(call_expr) => {
                                        let has_jsx_arg = call_expr.args.iter().any(|arg| {
                                            match &*arg.expr {
                                                Expr::Arrow(_) | Expr::Fn(_) => contains_jsx_in_expr(&arg.expr),
                                                _ => false
                                            }
                                        });
                                        
                                        if has_jsx_arg {
                                            let wrapped = Expr::Call(CallExpr {
                                                span: Default::default(),
                                                callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                                    observer_name.clone().into(),
                                                    Default::default(),
                                                    Default::default(),
                                                )))),
                                                args: vec![ExprOrSpread {
                                                    spread: None,
                                                    expr: init.clone(),
                                                }],
                                                type_args: None,
                                                ctxt: Default::default(),
                                            });
                                            *init = Box::new(wrapped);
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl)))
                },
                item => item,
            }
        }).collect();

        Module { body: transformed_body, ..module }
    }
}