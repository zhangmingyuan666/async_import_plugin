use swc_core::ecma::{
    ast::*,
    visit::VisitMut,
};
use swc_core::common::{
    Spanned, DUMMY_SP, Span,
    comments::{Comment, CommentKind, Comments},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use swc_core::atoms::Atom;
use swc_core::ecma::visit::VisitMutWith;

use serde::Deserialize;
use serde_json::Value;

use std::{
    path::Path,
    sync::{Mutex, Once},
    borrow::BorrowMut
};

mod shared;
pub use crate::shared::structs::MarkExpression;

// 全局计数器：用于为新组件分配递增 ID
static mut STD_ONCE_COUNTER: Option<Mutex<i64>> = None;
static INIT: Once = Once::new();

// 全局 JSON 缓存：缓存解析后的 map.json 内容
static mut JSON_VALUE: Option<Mutex<serde_json::Value>> = None;
static INIT_VALUE: Once = Once::new();

// 全局字符串 buffer：替代文件 I/O，在内存中暂存新组件映射
// 格式同 swc-chunk-pos.json: "{index}||{name}@@{index}||{name}@@..."
static mut CHUNK_POS_BUFFER: Option<Mutex<String>> = None;
static INIT_BUFFER: Once = Once::new();

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub record: Option<String>,
    pub noSplit: bool
}

fn global_string<'a>() -> &'a Mutex<i64> {
    INIT.call_once(|| {
        unsafe {
            *STD_ONCE_COUNTER.borrow_mut() = Some(Mutex::new(0));
        }
    });
    unsafe { STD_ONCE_COUNTER.as_ref().unwrap() }
}

fn global_map_json<'a>() -> &'a Mutex<serde_json::Value> {
    INIT_VALUE.call_once(|| {
        unsafe {
            *JSON_VALUE.borrow_mut() = Some(Mutex::new(Value::Null));
        }
    });
    unsafe { JSON_VALUE.as_ref().unwrap() }
}

fn global_buffer<'a>() -> &'a Mutex<String> {
    INIT_BUFFER.call_once(|| {
        unsafe {
            *CHUNK_POS_BUFFER.borrow_mut() = Some(Mutex::new(String::new()));
        }
    });
    unsafe { CHUNK_POS_BUFFER.as_ref().unwrap() }
}

impl<C: Comments> MarkExpression<C> {
    pub fn new(comments: C, config: &Config) -> Self {
        let record = config.record.to_owned().unwrap_or_default();
        let noSplit = config.noSplit;

        return Self {
            comments,
            record,
            noSplit
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
                // s1sAsyncImport 才需要进行处理
                if ident.sym == *"s1sAsyncImport" {
                    should_wrap = Some(true);

                    if let Some(args_first_element) = args.first() {
                        let expr = &*args_first_element.expr;
                        if let  Expr::Lit(Lit:: Str(Str {
                            value,
                            span,
                            ..
                        })) = expr {

                            let path_str = String::from_utf8_lossy(value.as_bytes()).to_string();
                            import_path = path_str.clone();
                            let path = Path::new(&path_str);

                            if let Some(file_stem) = path.file_stem() {
                                    let chunk_name = format!("{}", file_stem.to_str().unwrap());

                                    let chunk_name_copy = chunk_name.clone();

                                    let record_str = &self.record;

                                    // json 文件解析出的结果赋值在 v 对象上
                                    // 首次解析后缓存在全局变量中，后续直接使用缓存
                                    let mut v: serde_json::Value = Value::Null;
                                    let global_map_value = global_map_json().lock().unwrap().clone();

                                    if global_map_value == Value::Null {
                                        let res: serde_json::Value = serde_json::from_str(record_str).unwrap();
                                        v = res.clone();
                                        *global_map_json().lock().unwrap() = res;
                                    } else {
                                        v = global_map_value;
                                    }

                                    if let Some(jsChunkPos) = v.get("jsChunkPos") {
                                        // 缓存的最大值
                                        let max = jsChunkPos.get("max").unwrap();
                                        let max_i64 = max.as_i64();

                                        if let Some(dep) = jsChunkPos.get("dep") {
                                            if let Some(result) = dep.get(chunk_name) {
                                                // 已存在的组件：直接使用已有 index
                                                let index = result.as_i64().unwrap().to_string();

                                                let noSplitRef = self.noSplit;

                                                if noSplitRef {
                                                    comment_string = format!(" webpackChunkName: \"eager\" ");
                                                } else {
                                                    comment_string = format!(" webpackChunkName: \"{}-{}\" ",index,chunk_name_copy);
                                                }


                                            } else {
                                                // 新组件：分配递增 ID

                                                    let global_value = *global_string().lock().unwrap();

                                                    let mut max_value = 0 as i64;
                                                    if global_value == 0 {
                                                        max_value = max_i64.unwrap() + 1;
                                                    } else {
                                                        // 设置新的maxValue
                                                        max_value = global_value + 1;
                                                    }
                                                    // 设置全局计数器
                                                    *global_string().lock().unwrap() = max_value;

                                                    // 将新组件映射写入全局 buffer（替代文件 I/O，WASM 安全）
                                                    let buffer_string = format!("{}||{}@@", max_value, chunk_name_copy);
                                                    global_buffer().lock().unwrap().push_str(&buffer_string);

                                                    // 尝试写入 swc-chunk-pos.json（best-effort，WASM 中可能失败但不 panic）
                                                    if let Ok(current_dir) = std::env::current_dir() {
                                                        let map_path = current_dir.join("swc-chunk-pos.json");
                                                        if let Ok(mut file) = std::fs::OpenOptions::new()
                                                            .read(true)
                                                            .write(true)
                                                            .create(true)
                                                            .append(true)
                                                            .open(map_path) {
                                                            let _ = std::io::Write::write_fmt(&mut file, format_args!("{}", buffer_string.as_str()));
                                                        }
                                                    }

                                                    // 关键：将新组件同步写入全局缓存的 dep 中，避免重复分配 ID
                                                    {
                                                        let mut cached = global_map_json().lock().unwrap();
                                                        if let Some(js_chunk_pos) = cached.get_mut("jsChunkPos") {
                                                            // 更新 dep
                                                            if let Some(dep_obj) = js_chunk_pos.get_mut("dep") {
                                                                if let Some(obj) = dep_obj.as_object_mut() {
                                                                    obj.insert(chunk_name_copy.clone(), Value::Number(serde_json::Number::from(max_value)));
                                                                }
                                                            }
                                                            // 更新 max
                                                            if let Some(max_obj) = js_chunk_pos.get_mut("max") {
                                                                *max_obj = Value::Number(serde_json::Number::from(max_value));
                                                            }
                                                        }
                                                    }

                                                    // 组成「魔法注释」的字符串

                                                    let noSplitRef = self.noSplit;

                                                if noSplitRef {
                                                    comment_string = format!(" webpackChunkName: \"eager\" ");
                                                } else {
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

                // import 路径字符串的 AST 节点
                let import_node = ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(Str {
                        // dummy_with_cmt 包含上下文信息的 Dummy Span
                        span: Span::dummy_with_cmt(),
                        value: import_path.into(),
                        raw: None
                    })))
                };

                // import 路径字符串的 AST 节点
                let comment = Comment {
                    span: DUMMY_SP,
                    kind: CommentKind::Block,
                    text: comment_string.into()
                };

                // 加入注释
                self.comments.add_leading(import_node.span().hi, comment);

                // 赋值新 AST 结构
                *init = Box::new(Expr::Arrow(ArrowExpr {
                    span: DUMMY_SP,
                    ctxt: Default::default(),
                    params: vec![],
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                    body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                        span: DUMMY_SP,
                        ctxt: Default::default(),
                        stmts: vec![Stmt::Return(ReturnStmt {
                            span: DUMMY_SP,
                            arg: Some(Box::new(Expr::Call(CallExpr
                                {
                                    span: DUMMY_SP,
                                    ctxt: Default::default(),
                                    type_args: None,
                                    args: vec![ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(Expr::Arrow(ArrowExpr {
                                            span: DUMMY_SP,
                                            ctxt: Default::default(),
                                            is_async: false,
                                            is_generator: false,
                                            type_params: None,
                                            return_type: None,
                                            body: Box::new(Ident::new(Atom::from("res"), DUMMY_SP, Default::default()).into()),
                                            params: vec![Pat::Ident(BindingIdent {
                                                id: Ident::new(Atom::from("res"), DUMMY_SP, Default::default()),
                                                type_ann: None
                                            })],
                                        })),
                                    }],
                                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                                        span: DUMMY_SP,
                                        obj: Box::new(Expr::Call(CallExpr {
                                            type_args: None,
                                            span: DUMMY_SP,
                                            ctxt: Default::default(),
                                            callee: Callee::Import(Import {
                                                span: DUMMY_SP,
                                                phase: ImportPhase::Evaluation,
                                            }),
                                            args: vec![import_node],
                                        })),
                                        prop: MemberProp::Ident(IdentName::new(Atom::from("then"), DUMMY_SP)),
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

    let comments = metadata.comments.clone();

    program.visit_mut_with(&mut MarkExpression::new(comments, &config));

    program
}
