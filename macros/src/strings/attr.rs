pub trait AttrExt {
    fn is_emit_attr(&self) -> bool;
    fn is_skip_attr(&self) -> bool;
}

macro_rules! impl_is_attr {
    (
        $(
            $fn_ident:ident, $attr_name:expr,
        )*
    ) => {
        $(
            fn $fn_ident(&self) -> bool {
                let Ok(path) = self.meta.require_path_only() else {
                    return false;
                };
                let idents: Vec<_> = path.segments.iter().map(|s| &s.ident).collect();
                let [module_ident, ident] = *idents else {
                    return false;
                };

                module_ident == "strings" && ident == $attr_name
            }
        )*
    };
}

impl AttrExt for syn::Attribute {
    impl_is_attr! {
        is_emit_attr, "emit",
        is_skip_attr, "skip",
    }
}
