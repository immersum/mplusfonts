use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::visit_mut::*;
use syn::{Token, token};

use crate::strings::AttrExt;

pub struct MacroVisitor {
    bracket_token: token::Bracket,
    strings: Punctuated<String, Token![,]>,
    error: Option<syn::Error>,
}

impl MacroVisitor {
    pub fn new(strings: impl IntoIterator<Item = String>) -> Self {
        Self {
            bracket_token: Default::default(),
            strings: Punctuated::from_iter(strings),
            error: None,
        }
    }

    pub fn into_result(self) -> syn::Result<()> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn set_error(&mut self, error: syn::Error) {
        match self.error.as_mut() {
            Some(e) => e.combine(error),
            None => self.error = Some(error),
        }
    }

    fn append_strings_to_tokens(&self, tokens: &mut TokenStream) {
        if tokens.to_string().pop().is_some_and(|c| c != ',') {
            token::Comma::default().to_tokens(tokens);
        }

        self.bracket_token.surround(tokens, |tokens| {
            self.strings.to_tokens(tokens);
        });
    }

    fn expand_emit_attrs(&self, attrs: &[syn::Attribute], tokens: &mut TokenStream) {
        for _ in attrs.iter().filter(|attr| attr.is_emit_attr()) {
            self.append_strings_to_tokens(tokens);
        }
    }

    fn report_emit_attrs(&mut self, attrs: &[syn::Attribute]) {
        for attr in attrs.iter().filter(|attr| attr.is_emit_attr()) {
            let message = "`strings::emit` attribute can only be applied to \
                a macro invocation or its parent node";
            let error = syn::Error::new_spanned(attr, message);
            self.set_error(error);
        }
    }
}

macro_rules! impl_visit_macro_mut {
    (
        $(
            $fn_ident:ident, $node_type:ty,
        )*
    ) => {
        $(
            fn $fn_ident(&mut self, node: $node_type) {
                let attrs = &mut node.attrs;
                self.expand_emit_attrs(attrs, &mut node.mac.tokens);
                attrs.retain(|attr| !attr.is_emit_attr());
            }
        )*
    }
}

macro_rules! impl_visit_other_mut {
    (
        $(
            $fn_ident:ident, $node_ident:ident, $node_type:ty, $node_exprs:expr,
        )*
    ) => {
        $(
            fn $fn_ident(&mut self, $node_ident: $node_type) {
                let attrs = &mut $node_ident.attrs;
                let exprs = $node_exprs;
                let mut no_expr_macro = true;
                for expr in exprs {
                    if let syn::Expr::Macro(expr_macro) = expr {
                        no_expr_macro = false;
                        self.expand_emit_attrs(attrs, &mut expr_macro.mac.tokens);
                    }
                }
                if no_expr_macro {
                    self.report_emit_attrs(attrs);
                }

                attrs.retain(|attr| !attr.is_emit_attr());

                $fn_ident(self, $node_ident);
            }
        )*
    }
}

impl VisitMut for MacroVisitor {
    impl_visit_macro_mut! {
        visit_expr_macro_mut, &mut syn::ExprMacro,
        visit_foreign_item_macro_mut, &mut syn::ForeignItemMacro,
        visit_impl_item_macro_mut, &mut syn::ImplItemMacro,
        visit_item_macro_mut, &mut syn::ItemMacro,
        visit_stmt_macro_mut, &mut syn::StmtMacro,
        visit_trait_item_macro_mut, &mut syn::TraitItemMacro,
    }

    fn visit_attribute_mut(&mut self, node: &mut syn::Attribute) {
        if node.is_emit_attr() {
            let message = "`strings::emit` attribute is not supported here";
            let error = syn::Error::new_spanned(node, message);
            self.set_error(error);
        }
    }

    fn visit_attributes_mut(&mut self, attrs: &mut Vec<syn::Attribute>) {
        for attr in attrs.iter_mut() {
            self.visit_attribute_mut(attr);
        }

        attrs.retain(|attr| !attr.is_emit_attr());
    }

    impl_visit_other_mut! {
        visit_arm_mut, node, &mut syn::Arm,
            node.guard
                .as_mut_slice()
                .iter_mut()
                .map(|(_, expr)| expr.as_mut())
                .chain([node.body.as_mut()]),

        visit_const_param_mut, node, &mut syn::ConstParam,
            node.default.as_mut_slice(),

        visit_expr_array_mut, node, &mut syn::ExprArray,
            node.elems.iter_mut(),

        visit_expr_assign_mut, node, &mut syn::ExprAssign,
            [node.left.as_mut(), node.right.as_mut()],

        visit_expr_await_mut, node, &mut syn::ExprAwait,
            [node.base.as_mut()],

        visit_expr_binary_mut, node, &mut syn::ExprBinary,
            [node.left.as_mut(), node.right.as_mut()],

        visit_expr_break_mut, node, &mut syn::ExprBreak,
            node.expr
                .as_mut_slice()
                .iter_mut()
                .map(|expr| expr.as_mut()),

        visit_expr_call_mut, node, &mut syn::ExprCall,
            [node.func.as_mut()]
                .into_iter()
                .chain(node.args.iter_mut()),

        visit_expr_cast_mut, node, &mut syn::ExprCast,
            [node.expr.as_mut()],

        visit_expr_closure_mut, node, &mut syn::ExprClosure,
            [node.body.as_mut()],

        visit_expr_field_mut, node, &mut syn::ExprField,
            [node.base.as_mut()],

        visit_expr_for_loop_mut, node, &mut syn::ExprForLoop,
            [node.expr.as_mut()],

        visit_expr_group_mut, node, &mut syn::ExprGroup,
            [node.expr.as_mut()],

        visit_expr_if_mut, node, &mut syn::ExprIf,
            [node.cond.as_mut()]
                .into_iter()
                .chain(
                    node.else_branch
                        .as_mut()
                        .map(|(_, expr)| expr.as_mut()),
                ),

        visit_expr_index_mut, node, &mut syn::ExprIndex,
            [node.expr.as_mut()],

        visit_expr_let_mut, node, &mut syn::ExprLet,
            [node.expr.as_mut()],

        visit_expr_match_mut, node, &mut syn::ExprMatch,
            [node.expr.as_mut()],

        visit_expr_method_call_mut, node, &mut syn::ExprMethodCall,
            [node.receiver.as_mut()]
                .into_iter()
                .chain(node.args.iter_mut()),

        visit_expr_paren_mut, node, &mut syn::ExprParen,
            [node.expr.as_mut()],

        visit_expr_range_mut, node, &mut syn::ExprRange,
            node.start
                .as_deref_mut()
                .into_iter()
                .chain(node.end.as_deref_mut()),

        visit_expr_raw_addr_mut, node, &mut syn::ExprRawAddr,
            [node.expr.as_mut()],

        visit_expr_reference_mut, node, &mut syn::ExprReference,
            [node.expr.as_mut()],

        visit_expr_repeat_mut, node, &mut syn::ExprRepeat,
            [node.expr.as_mut(), node.len.as_mut()],

        visit_expr_return_mut, node, &mut syn::ExprReturn,
            node.expr
                .as_mut_slice()
                .iter_mut()
                .map(|expr| expr.as_mut()),

        visit_expr_struct_mut, node, &mut syn::ExprStruct,
            node.fields
                .iter_mut()
                .map(|field_value| &mut field_value.expr)
                .chain(node.rest.as_deref_mut()),

        visit_expr_try_mut, node, &mut syn::ExprTry,
            [node.expr.as_mut()],

        visit_expr_tuple_mut, node, &mut syn::ExprTuple,
            node.elems.iter_mut(),

        visit_expr_unary_mut, node, &mut syn::ExprUnary,
            [node.expr.as_mut()],

        visit_expr_while_mut, node, &mut syn::ExprWhile,
            [node.cond.as_mut()],

        visit_expr_yield_mut, node, &mut syn::ExprYield,
            node.expr
                .as_mut_slice()
                .iter_mut()
                .map(|expr| expr.as_mut()),

        visit_field_value_mut, node, &mut syn::FieldValue,
            [&mut node.expr],

        visit_impl_item_const_mut, node, &mut syn::ImplItemConst,
            [&mut node.expr],

        visit_item_const_mut, node, &mut syn::ItemConst,
            [node.expr.as_mut()],

        visit_item_enum_mut, node, &mut syn::ItemEnum,
            node.variants
                .iter_mut()
                .filter_map(|variant| {
                    variant
                        .discriminant
                        .as_mut()
                        .map(|(_, expr)| expr)
                }),

        visit_item_static_mut, node, &mut syn::ItemStatic,
            [node.expr.as_mut()],

        visit_local_mut, node, &mut syn::Local,
            node.init
                .as_mut_slice()
                .iter_mut()
                .flat_map(|local_init| {
                    [local_init.expr.as_mut()]
                        .into_iter()
                        .chain(
                            local_init
                                .diverge
                                .as_mut()
                                .map(|(_, expr)| expr.as_mut()),
                        )
                }),

        visit_trait_item_const_mut, node, &mut syn::TraitItemConst,
            node.default
                .as_mut_slice()
                .iter_mut()
                .map(|(_, expr)| expr),

        visit_variant_mut, node, &mut syn::Variant,
            node.discriminant
                .as_mut_slice()
                .iter_mut()
                .map(|(_, expr)| expr),
    }
}
