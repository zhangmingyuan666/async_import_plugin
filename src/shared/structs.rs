use swc_common::plugin::metadata;
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

pub struct MarkExpression<C: Comments> {
    pub comments: C,
}