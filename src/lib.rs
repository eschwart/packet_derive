use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(PacketEnum)]
pub fn into_packet(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Check if the input is an enum
    let variants = if let Data::Enum(data_enum) = input.data {
        data_enum.variants
    } else {
        return quote! { compile_error!("Unsupported structure (enum's only)") }.into();
        // Return empty if not an enum
    };

    let n = variants.len();

    let size = if n <= 8 {
        quote! { u8 }
    } else if n <= 16 {
        quote! { u16 }
    } else if n <= 32 {
        quote! { u32 }
    } else if n <= 64 {
        quote! { u64 }
    } else if n <= 128 {
        quote! { u128 }
    } else {
        return quote! { compile_error!("Enum has too many variants."); }.into();
    };

    let old_enum_name = input.ident;
    let new_enum_name = quote::format_ident!("{}Kind", old_enum_name);

    // Create the variants for the new enum (copied from the original)
    let flag_arms = variants.iter().enumerate().map(|(i, variant)| {
        let ident = &variant.ident;
        quote! {
            const #ident = 1 << #i;
        }
    });

    let match_arms = variants.iter().map(|variant| {
        let ident = &variant.ident;

        // Handle tuple and struct variants
        match &variant.fields {
            Fields::Unit => {
                quote! {
                    #old_enum_name::#ident => #new_enum_name::#ident,
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    #old_enum_name::#ident(..) => #new_enum_name::#ident,
                }
            }
            Fields::Named(_) => {
                quote! {
                    #old_enum_name::#ident { .. } => #new_enum_name::#ident,
                }
            }
        }
    });

    let doc_comment = format!(
        "Automatically generated bitflags for [`{}`].",
        old_enum_name
    );

    let new_enum = quote! {
        ::bitflags::bitflags! {
            #[doc = #doc_comment]
            #[derive(Clone, Copy, Debug)]
            pub struct #new_enum_name: #size {
                #(#flag_arms)*
            }
        }
    };

    let new_enum_impl = quote! {
        impl AsPacketKind for #new_enum_name {}

        impl AsPacketSend for #old_enum_name {}
        impl<'a> AsPacketRecv<'a, #new_enum_name> for #old_enum_name {
            fn kind(&self) -> #new_enum_name {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    // Build the new enum definition
    quote! {
        #new_enum
        #new_enum_impl

    }
    .into()
}
