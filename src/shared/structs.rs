use swc_core::ecma::{
    ast::*,
    visit::VisitMut,
};
use swc_core::common::{
    Spanned, DUMMY_SP, Span,
    comments::{Comment, CommentKind, Comments}
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use serde_json::Value;

pub struct MarkExpression<C: Comments> {
    pub comments: C,
    pub record: String,
    pub noSplit: bool
}
