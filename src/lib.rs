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

use serde::{de::value, Deserialize};
use serde_json::{Value, to_string_pretty, from_str, Map};

use std::{fmt::format, io::Read, path::Path};

use std::fs;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use std::sync::Mutex;
use std::sync::Once;
use std::borrow::BorrowMut;

static mut STD_ONCE_COUNTER: Option<Mutex<i64>> = None;
static INIT: Once = Once::new();
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub title: Option<String>,
    pub record: Option<String>
}

fn global_string<'a>() -> &'a Mutex<i64> {
    INIT.call_once(|| {
        // Since this access is inside a call_once, before any other accesses, it is safe
        unsafe {
            *STD_ONCE_COUNTER.borrow_mut() = Some(Mutex::new(0));
        }
    });
    // As long as this function is the only place with access to the static variable,
    // giving out a read-only borrow here is safe because it is guaranteed no more mutable
    // references will exist at this point or in the future.
    unsafe { STD_ONCE_COUNTER.as_ref().unwrap() }
}

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

                    if let Some(args_first_element) = args.first() {
                        let expr = &*args_first_element.expr;
                        if let  Expr::Lit(Lit:: Str(Str {
                            value,
                            span,
                            ..
                        })) = expr {        
                            let path_str = value.as_str();     
                            import_path = path_str.to_string();
        
                            let path = Path::new(&path_str);
        
                            if let Some(file_stem) = path.file_stem() {
                                    let chunk_name = format!("{}", file_stem.to_str().unwrap());
        
                                    let chunk_name_copy = chunk_name.clone();

                                    let record_str = &self.record;
                                    let v: serde_json::Value = serde_json::from_str(record_str).unwrap();
                                    
                                    if let Some(jsChunkPos) = v.get("jsChunkPos") {
                                        let max = jsChunkPos.get("max").unwrap();
                                        let max_i64 = max.as_i64();
                                        if let Some(dep) = jsChunkPos.get("dep") {        
                                            if let Some(result) = dep.get(chunk_name) {        
                                                let index = result.as_i64().unwrap().to_string();
        
                                                comment_string = format!(" webpackChunkName: \"{}-{}\" ",index,chunk_name_copy);
                                            } else {
                                                let current_dir = env::current_dir().unwrap();
                                                let map_path = current_dir.join("swc-chunk-pos.json");

                                                let mut file = OpenOptions::new()
                                                    .read(true)
                                                    .write(true)
                                                    .create(true)
                                                    .append(true)
                                                    .open(map_path)
                                                    .expect("Failed to open or create the file");

                                                    let global_value = *global_string().lock().unwrap();
                                                    // 首次获取到新组件，需要写入到json文件内
                                                    if global_value == 0 {
                        
                                                        let max_value = max_i64.unwrap() + 1;
                                                        // 设置新的maxValue
                                                        *global_string().lock().unwrap() = max_value;

                                                        let file_insert_string = format!("{}||{}@@",max_value.to_string(),chunk_name_copy);

                                                        write!(file, "{}", file_insert_string.as_str());

                                                        comment_string = format!(" webpackChunkName: \"{}-{}\" ",max_value.to_string(),chunk_name_copy);
                                                    } else {
                                                        // 设置新的maxValue
                                                        let max_value = global_value + 1;

                                                        *global_string().lock().unwrap() = max_value;

                                                        let file_insert_string = format!("{}||{}@@",max_value.to_string(),chunk_name_copy);

                                                        write!(file, "{}", file_insert_string.as_str());

                                                        comment_string = format!(" webpackChunkName: \"{}-{}\" ",max_value.to_string(),chunk_name_copy);
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

        

        match should_wrap {
            // 应该进行处理
            Some(true) => {
                let init: &mut Box<Expr> = e.init.as_mut().unwrap();

                let import_node = ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str((Str {
                        span: Span::dummy_with_cmt(),
                        value: import_path.into(),
                        raw: None
                    }))))
                };

                let comment = Comment {
                    span: DUMMY_SP,
                    kind: CommentKind::Block,
                    text: comment_string.into()
                };
                self.comments.add_leading(import_node.span().hi, comment);

                *init = Box::new(Expr::Arrow(ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![],
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                    body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                        span: DUMMY_SP,
                        stmts: vec![Stmt::Return(ReturnStmt {
                            span: DUMMY_SP,
                            arg: Some(Box::new(Expr::Call(CallExpr
                                {
                                    span: DUMMY_SP,
                                    type_args: None,
                                    args: vec![ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Arrow(ArrowExpr {
                                            span: DUMMY_SP,                                           
                                            is_async: false,
                                            is_generator: false,
                                            type_params: None,
                                            return_type: None,
                                            body: Box::new(Ident::new(JsWord::from("res"), DUMMY_SP).into()),
                                            params: vec![Pat::Ident(BindingIdent {
                                                id: Ident {
                                                    span: DUMMY_SP,
                                                    sym: JsWord::from("res"),
                                                    optional: false
                                                },
                                                type_ann: None
                                            })],
                                        })),
                                    }],
                                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                                        span: DUMMY_SP,
                                        obj: Box::new(Expr::Call(CallExpr {
                                            type_args: None,
                                            span: DUMMY_SP,
                                            callee: Callee::Import(Import {
                                                span: DUMMY_SP,
                                                phase: ImportPhase::Evaluation,
                                            }),
                                            args: vec![import_node],
                                        })),
                                        prop: MemberProp::Ident(quote_ident!("then")),
                                })))
                                    
                                    

                                }
                            )))
                        }) ]
                    }))
                }));                 
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