use std::panic;

use proc_macro2::Span;
use quote::format_ident;

pub trait ItemExt {
    fn variant_ident(&self) -> syn::Ident;
}

impl ItemExt for syn::Item {
    fn variant_ident(&self) -> syn::Ident {
        use syn::Item::*;

        match self {
            Const(syn::ItemConst { ident, .. }) => {
                format_ident!("const_{ident}")
            }
            Enum(syn::ItemEnum { ident, .. }) => {
                format_ident!("enum_{ident}")
            }
            ExternCrate(syn::ItemExternCrate { ident, .. }) => {
                format_ident!("extern_crate_{ident}")
            }
            Fn(syn::ItemFn {
                sig: syn::Signature { ident, .. },
                ..
            }) => {
                format_ident!("fn_{ident}")
            }
            ForeignMod(syn::ItemForeignMod {
                abi: syn::Abi { name, .. },
                ..
            }) => {
                let ident = name.as_ref().map(|n| {
                    panic::catch_unwind(|| syn::Ident::new_raw(&n.value(), Span::call_site())).ok()
                });
                if let Some(ident) = ident.flatten() {
                    format_ident!("extern_{ident}")
                } else {
                    format_ident!("extern")
                }
            }
            Impl(syn::ItemImpl {
                trait_, self_ty, ..
            }) => {
                let trait_ident = if let Some((_, path, _)) = trait_.as_ref() {
                    Some(path.segments.last().map(|s| &s.ident))
                } else {
                    None
                };
                let ident = if let syn::Type::Path(syn::TypePath { path, .. }) = self_ty.as_ref() {
                    Some(path.segments.last().map(|s| &s.ident))
                } else {
                    None
                };
                match (trait_ident.flatten(), ident.flatten()) {
                    (Some(trait_ident), Some(ident)) => {
                        format_ident!("impl_{trait_ident}_for_{ident}")
                    }
                    (Some(trait_ident), None) => {
                        format_ident!("impl_{trait_ident}")
                    }
                    (None, Some(ident)) => {
                        format_ident!("impl_{ident}")
                    }
                    (None, None) => {
                        format_ident!("impl")
                    }
                }
            }
            Macro(syn::ItemMacro {
                ident,
                mac: syn::Macro { path, .. },
                ..
            }) => {
                let macro_ident = path.segments.last().map(|s| &s.ident);
                match (macro_ident, ident) {
                    (Some(_), Some(ident)) if path.is_ident("macro_rules") => {
                        format_ident!("macro_rules_{ident}")
                    }
                    (Some(macro_ident), Some(ident)) => {
                        format_ident!("{macro_ident}_{ident}")
                    }
                    (Some(macro_ident), None) => {
                        format_ident!("{macro_ident}")
                    }
                    (None, Some(ident)) => {
                        format_ident!("macro_{ident}")
                    }
                    (None, None) => {
                        format_ident!("macro")
                    }
                }
            }
            Mod(syn::ItemMod { ident, .. }) => {
                format_ident!("mod_{ident}")
            }
            Static(syn::ItemStatic { ident, .. }) => {
                format_ident!("static_{ident}")
            }
            Struct(syn::ItemStruct { ident, .. }) => {
                format_ident!("struct_{ident}")
            }
            Trait(syn::ItemTrait { ident, .. }) => {
                format_ident!("trait_{ident}")
            }
            TraitAlias(syn::ItemTraitAlias { ident, .. }) => {
                format_ident!("trait_{ident}")
            }
            Type(syn::ItemType { ident, .. }) => {
                format_ident!("type_{ident}")
            }
            Union(syn::ItemUnion { ident, .. }) => {
                format_ident!("union_{ident}")
            }
            Use(syn::ItemUse { .. }) => {
                format_ident!("use")
            }
            Verbatim(_) | _ => {
                format_ident!("item")
            }
        }
    }
}
