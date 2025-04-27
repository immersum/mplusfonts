use proc_macro2::{TokenStream, TokenTree};
use syn::visit_mut::*;

use crate::strings::AttrExt;

pub struct LitStrVisitor {
    strings: Vec<String>,
    error: Option<syn::Error>,
}

impl LitStrVisitor {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            error: None,
        }
    }

    pub fn into_result(self) -> syn::Result<Vec<String>> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(self.strings),
        }
    }

    fn set_error(&mut self, error: syn::Error) {
        match self.error.as_mut() {
            Some(e) => e.combine(error),
            None => self.error = Some(error),
        }
    }
}

macro_rules! impl_visit_other_mut {
    (
        $(
            $fn_ident:ident, $node_type:ty,
        )*
    ) => {
        $(
            fn $fn_ident(&mut self, node: $node_type) {
                let attrs = &mut node.attrs;
                let original_len = attrs.len();

                attrs.retain(|attr| !attr.is_skip_attr());

                if attrs.len() == original_len {
                    $fn_ident(self, node);
                }
            }
        )*
    }
}

impl VisitMut for LitStrVisitor {
    fn visit_lit_str_mut(&mut self, node: &mut syn::LitStr) {
        self.strings.push(node.value());
    }

    fn visit_token_stream_mut(&mut self, tokens: &mut TokenStream) {
        for tt in tokens.clone() {
            if let TokenTree::Literal(token) = tt {
                self.visit_lit_mut(&mut syn::Lit::new(token));
            } else if let TokenTree::Group(token) = tt {
                self.visit_token_stream_mut(&mut token.stream());
            }
        }
    }

    fn visit_attribute_mut(&mut self, node: &mut syn::Attribute) {
        if node.is_skip_attr() {
            let message = "`strings::skip` attribute is not supported here";
            let error = syn::Error::new_spanned(node, message);
            self.set_error(error);
        }
    }

    fn visit_attributes_mut(&mut self, attrs: &mut Vec<syn::Attribute>) {
        for attr in attrs.iter_mut() {
            self.visit_attribute_mut(attr);
        }

        attrs.retain(|attr| !attr.is_skip_attr());
    }

    impl_visit_other_mut! {
        visit_arm_mut, &mut syn::Arm,
        visit_bare_fn_arg_mut, &mut syn::BareFnArg,
        visit_bare_variadic_mut, &mut syn::BareVariadic,
        visit_const_param_mut, &mut syn::ConstParam,
        visit_derive_input_mut, &mut syn::DeriveInput,
        visit_expr_array_mut, &mut syn::ExprArray,
        visit_expr_assign_mut, &mut syn::ExprAssign,
        visit_expr_async_mut, &mut syn::ExprAsync,
        visit_expr_await_mut, &mut syn::ExprAwait,
        visit_expr_binary_mut, &mut syn::ExprBinary,
        visit_expr_block_mut, &mut syn::ExprBlock,
        visit_expr_break_mut, &mut syn::ExprBreak,
        visit_expr_call_mut, &mut syn::ExprCall,
        visit_expr_cast_mut, &mut syn::ExprCast,
        visit_expr_closure_mut, &mut syn::ExprClosure,
        visit_expr_const_mut, &mut syn::ExprConst,
        visit_expr_continue_mut, &mut syn::ExprContinue,
        visit_expr_field_mut, &mut syn::ExprField,
        visit_expr_for_loop_mut, &mut syn::ExprForLoop,
        visit_expr_group_mut, &mut syn::ExprGroup,
        visit_expr_if_mut, &mut syn::ExprIf,
        visit_expr_index_mut, &mut syn::ExprIndex,
        visit_expr_infer_mut, &mut syn::ExprInfer,
        visit_expr_let_mut, &mut syn::ExprLet,
        visit_expr_lit_mut, &mut syn::ExprLit,
        visit_expr_loop_mut, &mut syn::ExprLoop,
        visit_expr_macro_mut, &mut syn::ExprMacro,
        visit_expr_match_mut, &mut syn::ExprMatch,
        visit_expr_method_call_mut, &mut syn::ExprMethodCall,
        visit_expr_paren_mut, &mut syn::ExprParen,
        visit_expr_path_mut, &mut syn::ExprPath,
        visit_expr_range_mut, &mut syn::ExprRange,
        visit_expr_raw_addr_mut, &mut syn::ExprRawAddr,
        visit_expr_reference_mut, &mut syn::ExprReference,
        visit_expr_repeat_mut, &mut syn::ExprRepeat,
        visit_expr_return_mut, &mut syn::ExprReturn,
        visit_expr_struct_mut, &mut syn::ExprStruct,
        visit_expr_try_mut, &mut syn::ExprTry,
        visit_expr_try_block_mut, &mut syn::ExprTryBlock,
        visit_expr_tuple_mut, &mut syn::ExprTuple,
        visit_expr_unary_mut, &mut syn::ExprUnary,
        visit_expr_unsafe_mut, &mut syn::ExprUnsafe,
        visit_expr_while_mut, &mut syn::ExprWhile,
        visit_expr_yield_mut, &mut syn::ExprYield,
        visit_field_mut, &mut syn::Field,
        visit_field_pat_mut, &mut syn::FieldPat,
        visit_field_value_mut, &mut syn::FieldValue,
        visit_file_mut, &mut syn::File,
        visit_foreign_item_fn_mut, &mut syn::ForeignItemFn,
        visit_foreign_item_macro_mut, &mut syn::ForeignItemMacro,
        visit_foreign_item_static_mut, &mut syn::ForeignItemStatic,
        visit_foreign_item_type_mut, &mut syn::ForeignItemType,
        visit_impl_item_const_mut, &mut syn::ImplItemConst,
        visit_impl_item_fn_mut, &mut syn::ImplItemFn,
        visit_impl_item_macro_mut, &mut syn::ImplItemMacro,
        visit_impl_item_type_mut, &mut syn::ImplItemType,
        visit_item_const_mut, &mut syn::ItemConst,
        visit_item_enum_mut, &mut syn::ItemEnum,
        visit_item_extern_crate_mut, &mut syn::ItemExternCrate,
        visit_item_fn_mut, &mut syn::ItemFn,
        visit_item_foreign_mod_mut, &mut syn::ItemForeignMod,
        visit_item_impl_mut, &mut syn::ItemImpl,
        visit_item_macro_mut, &mut syn::ItemMacro,
        visit_item_mod_mut, &mut syn::ItemMod,
        visit_item_static_mut, &mut syn::ItemStatic,
        visit_item_struct_mut, &mut syn::ItemStruct,
        visit_item_trait_mut, &mut syn::ItemTrait,
        visit_item_trait_alias_mut, &mut syn::ItemTraitAlias,
        visit_item_type_mut, &mut syn::ItemType,
        visit_item_union_mut, &mut syn::ItemUnion,
        visit_item_use_mut, &mut syn::ItemUse,
        visit_lifetime_param_mut, &mut syn::LifetimeParam,
        visit_local_mut, &mut syn::Local,
        visit_pat_ident_mut, &mut syn::PatIdent,
        visit_pat_or_mut, &mut syn::PatOr,
        visit_pat_paren_mut, &mut syn::PatParen,
        visit_pat_reference_mut, &mut syn::PatReference,
        visit_pat_rest_mut, &mut syn::PatRest,
        visit_pat_slice_mut, &mut syn::PatSlice,
        visit_pat_struct_mut, &mut syn::PatStruct,
        visit_pat_tuple_mut, &mut syn::PatTuple,
        visit_pat_tuple_struct_mut, &mut syn::PatTupleStruct,
        visit_pat_type_mut, &mut syn::PatType,
        visit_pat_wild_mut, &mut syn::PatWild,
        visit_receiver_mut, &mut syn::Receiver,
        visit_stmt_macro_mut, &mut syn::StmtMacro,
        visit_trait_item_const_mut, &mut syn::TraitItemConst,
        visit_trait_item_fn_mut, &mut syn::TraitItemFn,
        visit_trait_item_macro_mut, &mut syn::TraitItemMacro,
        visit_trait_item_type_mut, &mut syn::TraitItemType,
        visit_type_param_mut, &mut syn::TypeParam,
        visit_variadic_mut, &mut syn::Variadic,
        visit_variant_mut, &mut syn::Variant,
    }
}
