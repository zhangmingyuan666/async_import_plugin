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

use serde::Deserialize;
use serde_json::{Value, to_string_pretty, from_str, Map};

use std::{fmt::format, path::Path};


#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub title: Option<String>,
    pub record: Option<String>
}

/*
fn update_js_chunk_pos(record: &Value, chunk_name: &str) -> Option<i32> {
    let js_chunk_pos = record.get("jsChunkPos").expect("jsChunkPos field not found");

    // let dep = js_chunk_pos.get("dep").expect("dep field not found");

    // if let Some(Value::Number(index)) = dep.get(chunk_name) {
    //     let number = index.as_i64()?;
    //     println!("-------- ddddddddd {:?}", number);
    //     return Some(number as i32)
    // } else {
    //     println!("-------- ddddddddd");

    //     return None
    // }
    /* 
    let dep = js_chunk_pos
        .get_mut("dep")
        .expect("dep field not found")
        .as_object_mut()
        .expect("dep is not an object");

    // let max = js_chunk_pos
    //     .get_mut("max").expect("1213").as_u64().expect("1213");

    if let Some(Value::Number(index)) = dep.get(chunk_name) {
        // let number = index.as_i64()?;
        return Some(number as i32)
    } else {

        // let new_max = max + 1;
        // dep.insert(chunk_name.to_owned(), Value::Number(serde_json::Number::from(new_max)));

        // Some(new_max as i32)
    }
    */
    return None
}
*/   

impl<C: Comments> MarkExpression<C> {
    pub fn new(comments: C, config: &Config) -> Self {

        let title: String = config.title.to_owned().unwrap_or_default();

        let record = config.record.to_owned().unwrap_or_default();

        return Self {
            comments,
            title,
            record
        }
    }
}


impl<C: Comments> VisitMut for MarkExpression<C> {
    fn visit_mut_var_declarator(&mut self, e: &mut VarDeclarator) {        
        let mut import_path = String::from("");
        
        // e.visit_mut_children_with(self);

        let mut comment_string = String::from("");

        let mut should_wrap: Option<bool> = Some(false);
        if let Some(Expr::Call(CallExpr {
            callee: Callee::Expr(callee),
            args,
            ..
        })) = e.init.as_deref().as_mut()
        {
            if let Expr::Ident(ident) = &**callee {
                // 如果发现是此函数，要给
                if ident.sym == *"s1sAsyncImport" {
                    should_wrap = Some(true);
                }   
            }

            if let Some(args_first_element) = args.first() {
                let expr = &*args_first_element.expr;

                if let  Expr::Lit(Lit:: Str(Str {
                    value,
                    ..
                })) = expr {
                    let path_str = value.as_str();

                    import_path = path_str.to_string();

                    let path = Path::new(&path_str);
                    if let Some(file_stem) = path.file_stem() {
                        if let Some(extension) = path.extension() {
                            let chunk_name = format!("{}", file_stem.to_str().unwrap());

                            let chunk_name_copy = chunk_name.clone();

                            let record_str = &self.record;
                            let v: serde_json::Value = serde_json::from_str(record_str).unwrap();
                            
                            if let Some(jsChunkPos) = v.get("jsChunkPos") {
                                if let Some(dep) = jsChunkPos.get("dep") {
                                    // println!("--- in dep ---- ");

                                    if let Some(result) = dep.get(chunk_name) {
                                        println!("successful ----{:?} ", result);

                                        let index = result.as_i64().unwrap().to_string();

                                        comment_string = format!(" webpackChunkName: {}-{} ",index,chunk_name_copy);

                                        // println!("{:?}", new_string);

                                        // chunk_name
                                    } else {
                                        // println!("fail ----");
                                    }
                                }
                            }
                            

                            let comment = Comment {
                                span: DUMMY_SP,
                                kind: CommentKind::Block,
                                text: comment_string.into(),
                            };
                            self.comments.add_trailing(e.span.hi, comment);
                    
                            //let indexOption = update_js_chunk_pos(&record, chunk_name.as_str());

                            // if let Some(index) = indexOption {
                            //     println!("indexindexindex{:?}", index);
                            // }


                            // 去record.jsChunkPos.dep中取chunkName的value
                            // 如果存在
                                // index 设置为这个缓存
                            // 如果不存在
                                // 设置 record.jsChunkPos.max + 1

                            


                        }
                    }

                    
                }
            }
        }

        

        match should_wrap {
            // 应该进行处理
            Some(true) => {
                let init: &mut Box<Expr> = e.init.as_mut().unwrap();

        
                let origin_span = e.span;

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
                                                    value: import_path.into(),
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
                // comments 
                // self.comments.add_trailing(newInit.span_hi(), comment);
                
            }

            // 无需进行处理
            Some(false) => {
            }

            _ => {
            }
        }
    
    }
}

#[plugin_transform]
pub fn process_transform(mut program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config_str = &metadata
        .get_transform_plugin_config()
        .expect("Failed to resolve config");

    let config: Config = serde_json::from_str::<Option<Config>>(config_str.as_str()).expect("Invalid config")
    .unwrap();
    
    let comments = match metadata.comments {
        Some(comments) => comments.clone(),
        None => PluginCommentsProxy,
    };
    
    program.visit_mut_with(&mut MarkExpression::new(comments, &config));

    program
}