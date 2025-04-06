use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};

pub struct Image {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub data: Vec<u8>,
}

pub struct ImageList(pub Vec<Image>);

impl ToTokens for Image {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            left,
            top,
            width,
            data,
        } = self;

        let data = Literal::byte_string(data);
        let image_raw = quote! {
            ::mplusfonts::image::ImageRaw::new(#data, #width)
        };
        let offset = quote! {
            ::embedded_graphics::geometry::Point::new(#left, #top)
        };
        let image = quote! {
            ::mplusfonts::image::Image::new(#image_raw, #offset)
        };

        tokens.extend(image);
    }
}

impl ToTokens for ImageList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(vec) = self;

        debug_assert!(
            !vec.is_empty(),
            "expected image list to contain at least one render"
        );
        let image_set = match vec.as_slice() {
            [] => quote! {
                ::mplusfonts::image::ImageSet::Repeated(::mplusfonts::image::Image::NULL)
            },
            [image] => quote! {
                ::mplusfonts::image::ImageSet::Repeated(#image)
            },
            _ => quote! {
                ::mplusfonts::image::ImageSet::Array([#(#vec),*])
            },
        };

        tokens.extend(image_set);
    }
}
