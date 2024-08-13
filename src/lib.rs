use swc_common::{plugin::metadata, SyntaxContext};
use swc_core::ecma::{
    ast::*,
    transforms::testing::test_inline,
    visit::{as_folder, FoldWith, VisitMut},
};
use swc_common::{
    BytePos, SourceMapperDyn, Spanned, DUMMY_SP, Span,
    comments::{Comment, CommentKind, Comments}
};
use swc_core::plugin::{plugin_transform, proxies::{TransformPluginProgramMetadata, PluginCommentsProxy}};

mod shared;
pub use crate::shared::structs::MarkExpression;

use swc_ecma_visit::VisitMutWith;
use swc_core::ecma::atoms::JsWord;
use swc_ecma_utils::{quote_ident};

impl<C: Comments> MarkExpression<C> {
    pub fn new(comments: C) -> Self {
        return Self {
            comments
        }
    }
}


impl<C: Comments> VisitMut for MarkExpression<C> {
    fn visit_mut_var_declarator(&mut self, e: &mut VarDeclarator) {
        
        e.visit_mut_children_with(self);

        print!("start{:?},end:{:?}", e.span.lo, e.span.hi);
  
        let comment = Comment {
            span: DUMMY_SP,
            kind: CommentKind::Block,
            text: " webpackChunkName: 0-bundle ".into(),
        };

        let mut should_wrap = Some(false);
        if let Some(Expr::Call(CallExpr {
            callee: Callee::Expr(callee),
            args,
            ..
        })) = e.init.as_deref().as_mut()
        {
            if let Expr::Ident(ident) = &**callee {
                // 如果发现是此函数，要给
                if ident.sym == *"s1sAsyncImport" {
                    should_wrap = Some(true)
                }   
            }
        }

        

        match should_wrap {
            // 应该进行处理
            Some(true) => {
                println!("True");

                let init: &mut Box<Expr> = e.init.as_mut().unwrap();

                

                // self.comments.add_trailing(e.span.hi, comment);
        

                


                let origin_span = e.span;

                println!("------------000000000{:?}", origin_span.hi);

                *init = Box::new(Expr::Arrow(ArrowExpr {
                    span: origin_span,
                    params: vec![],
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                    body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                        span: origin_span,
                        stmts: vec![Stmt::Return(ReturnStmt {
                            span: origin_span,
                            arg: Some(Box::new(Expr::Call(CallExpr
                                {
                                    span: origin_span,
                                    type_args: None,
                                    args: vec![ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Arrow(ArrowExpr {
                                            span: origin_span,                                           
                                            is_async: false,
                                            is_generator: false,
                                            type_params: None,
                                            return_type: None,
                                            body: Box::new(Ident::new(JsWord::from("res"), origin_span).into()),
                                            params: vec![Pat::Ident(BindingIdent {
                                                id: Ident {
                                                    span: origin_span,
                                                    sym: JsWord::from("res"),
                                                    optional: false
                                                },
                                                type_ann: None
                                            })],
                                        })),
                                    }],
                                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                                        span: origin_span,
                                        obj: Box::new(Expr::Call(CallExpr {
                                            type_args: None,
                                            span: origin_span,
                                            callee: Callee::Import(Import {
                                                span: origin_span,
                                                phase: ImportPhase::Evaluation,
                                            }),
                                            args: vec![ExprOrSpread {
                                                spread: None,
                                                expr: Box::new(Expr::Lit(Lit::Str((Str {
                                                    span: origin_span,
                                                    value: "1".into(),
                                                    raw: None
                                                }))))
                                            }],
                                        })),
                                        prop: MemberProp::Ident(quote_ident!("then")),
                                })))
                                    
                                    

                                }
                            )))
                        }) ]
                    }))
                }));
            
             
                let mut span = e.span.hi;
                

                let newInit = &**init;
                self.comments.add_trailing(newInit.span_hi(), comment);
                print!("After handle start{:?},end:{:?}", newInit.span().lo, newInit.span().hi);

                
                if let Expr::Arrow(ArrowExpr { body, .. }) = &*init.clone() {
                    if let BlockStmtOrExpr::BlockStmt(BlockStmt { stmts, .. }) = &**body {
                        if let Some(first) = stmts.first() {
                            // span = first.span().hi;
                            if let Stmt::Return(ReturnStmt { arg, .. }) = &*first {
                                if let Some(arg) = arg {
                                    if let Expr::Call(CallExpr {
                                        callee,
                                        ..
                                    }) = &**arg {
                                        if let Callee::Expr(test_item) = callee {
                                       
                                            if let Expr::Member(MemberExpr {
                                                obj,
                                                ..
                                            }) = &**test_item
                                            {
                                                if let Expr::Call(CallExpr {
                                                    callee,
                                                    args,
                                                    ..
                                                }) = &**obj
                                                {
                                                    if let Some(first) = args.first()
                                                    {
                                                        

                                                        if let ExprOrSpread {
                                                            expr,
                                                            ..
                                                        }  = first
                                                        {
                                                            

                                                            if let Expr::Lit(Lit::Str(Str { span, .. })) = &**expr {
                                                                println!("-21-3-21-122-13-123-123-12");

                                                                let comment = Comment {
                                                                    span: DUMMY_SP,
                                                                    kind: CommentKind::Block,
                                                                    text: " webpackChunkName: 0-bundle ".into(),
                                                                };

                                                                println!("cajkscakjlcakc {:?}  acjklsclkaj", span);
                                            
                                                                // 添加注释到字符串字面量
                                                                //self.comments.add_leading(span.lo, comment);
                                                            
                                                            }
                                                        }
                                                    }
                                                    
                                                }
                                                
                     
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            
                
            }

            // 无需进行处理
            Some(false) => {
                println!("False");
            }

            _ => {
                println!("False");
            }
        }
    
    }
}



/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually
/// if plugin need to handle low-level ptr directly via
/// `__transform_plugin_process_impl(
///     ast_ptr: *const u8, ast_ptr_len: i32,
///     unresolved_mark: u32, should_enable_comments_proxy: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */`
///
/// This requires manual handling of serialization / deserialization from ptrs.
/// Refer swc_plugin_macro to see how does it work internally.
#[plugin_transform]
pub fn process_transform(mut program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let comments = match metadata.comments {
        Some(comments) => comments.clone(),
        None => PluginCommentsProxy,
    };
    
    program.visit_mut_with(&mut MarkExpression::new(comments));

    program
}