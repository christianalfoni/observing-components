use swc_ecma_ast::*;
use swc_ecma_visit::{fold_pass, noop_fold_type, Fold};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub import_name: Option<String>,
    pub import_path: String,
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
        // NEW: Check if Call expressions' arguments contain JSX
        Expr::Call(call_expr) => call_expr.args.iter().any(|arg| contains_jsx_in_expr(&arg.expr)),
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
        _ => false
    }
}

fn contains_jsx_in_module(module: &Module) -> bool {
    module.body.iter().any(|item| match item {
        ModuleItem::Stmt(stmt) => contains_jsx_in_stmt(stmt),
        ModuleItem::ModuleDecl(decl) => match decl {
            ModuleDecl::ExportDefaultExpr(export) => contains_jsx_in_expr(&export.expr),
            ModuleDecl::ExportDecl(export_decl) => {
                if let Decl::Fn(fn_decl) = &export_decl.decl {
                    contains_jsx_in_function(&fn_decl.function)
                } else {
                    false
                }
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

impl Fold for ObserverTransform {
    noop_fold_type!();

    fn fold_module(&mut self, mut module: Module) -> Module {
        // Use the new helper to check if any function with JSX is present.
        let should_add_import = contains_jsx_in_module(&module);
        if should_add_import && !self.has_added_import {
            // Add import statement with config values.
            let import_name = self.get_import_name();
            let import_path = self.config.import_path.clone();
            let import = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: Default::default(),
                specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                    span: Default::default(),
                    local: Ident::new(import_name.clone().into(), Default::default(), Default::default()),
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

        // Transform the module items
        let transformed_body = module.body.into_iter().map(|item| {
            let import_name = self.get_import_name();
            match item {
                // NEW: Handle non-exported function declarations containing JSX
                ModuleItem::Stmt(Stmt::Decl(Decl::Fn(fn_decl))) => {
                    if contains_jsx_in_function(&fn_decl.function) {
                        let ident = fn_decl.ident.clone();
                        let fn_expr = Expr::Fn(FnExpr {
                            ident: Some(ident.clone()),
                            function: fn_decl.function.clone(),
                        });
                        let wrapped_fn_expr = Expr::Call(CallExpr {
                            span: Default::default(),
                            callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                import_name.clone().into(),
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
                            import_name.clone().into(),
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
                // NEW: Transform variable declarations (e.g., const Home2 = () => {...}) with arrow functions containing JSX.
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(mut export_decl)) => {
                    match &mut export_decl.decl {
                        // NEW: Transform function declarations into const variable declarations and wrap with observer
                        Decl::Fn(fn_decl) => {
                            if contains_jsx_in_function(&fn_decl.function) {
                                let ident = fn_decl.ident.clone();
                                let fn_expr = Expr::Fn(FnExpr {
                                    ident: Some(ident.clone()),
                                    function: fn_decl.function.clone(),
                                });
                                // NEW: Wrap the function expression with observer
                                let wrapped_fn_expr = Expr::Call(CallExpr {
                                    span: Default::default(),
                                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                        import_name.clone().into(),
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
                        // EXISTING: Handling of variable declarations and other cases
                        Decl::Var(var_decl) => {
                            // ...existing code for handling var declarations...
                            for decl in var_decl.decls.iter_mut() {
                                if let Some(init) = &mut decl.init {
                                    // NEW: Handle wrapping functions inside any call expression (not just "memo")
                                    if let Expr::Call(call_expr) = &mut **init {
                                        if let Some(arg) = call_expr.args.get_mut(0) {
                                            match &mut *arg.expr {
                                                Expr::Fn(_) | Expr::Arrow(_) => {
                                                    let wrapped = Expr::Call(CallExpr {
                                                        span: Default::default(),
                                                        callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                                            import_name.clone().into(),
                                                            Default::default(),
                                                            Default::default(),
                                                        )))),
                                                        args: vec![ExprOrSpread {
                                                            spread: None,
                                                            expr: arg.expr.clone(),
                                                        }],
                                                        type_args: None,
                                                        ctxt: Default::default(),
                                                    });
                                                    arg.expr = Box::new(wrapped);
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                    // EXISTING: Handle arrow functions
                                    match &mut **init {
                                        Expr::Arrow(_) => {
                                            if contains_jsx_in_expr(&*init) {
                                                let wrapped = Expr::Call(CallExpr {
                                                    span: Default::default(),
                                                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                                        import_name.clone().into(),
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
                                        // EXISTING: Handle function expressions
                                        Expr::Fn(f_expr) => {
                                            if contains_jsx_in_function(&f_expr.function) {
                                                let wrapped = Expr::Call(CallExpr {
                                                    span: Default::default(),
                                                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                                        import_name.clone().into(),
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
                            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl))
                        },
                        // ...existing code...
                        _ => ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl))
                    }
                },
                // NEW: Transform default export declarations with functions containing JSX
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl)) => {
                    if let swc_ecma_ast::DefaultDecl::Fn(ref f) = export_decl.decl {
                        if contains_jsx_in_function(&f.function) {
                            let wrapped_expr = Expr::Call(CallExpr {
                                span: Default::default(),
                                callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                                    import_name.clone().into(),
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
                item => item,
            }
        }).collect();

        Module { body: transformed_body, ..module }
    }
}