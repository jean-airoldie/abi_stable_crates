/*!
An implementation detail of abi_stable.
*/

#![recursion_limit="192"]
// #![deny(unused_variables)]
// #![deny(unused_imports)]
// #![deny(unused_parens)]
// #![deny(unused_assignments)]
// #![deny(unused_mut)]
#![deny(unreachable_patterns)]
#![deny(unused_doc_comments)]
#![deny(unconditional_recursion)]


extern crate proc_macro;

/**


This macro is documented in abi_stable::docs::stable_abi_derive

*/

#[proc_macro_derive(StableAbi, attributes(sabi))]
pub fn derive_stable_abi(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( input, stable_abi::derive ).into()
}




/**

Allows implementing the InterfaceType trait,
providing default values for associated types not specified in the impl block.

<b>
This macro has been deprecated in favor of using the `#[sabi(impl_InterfaceType())]` 
helper attribute of both `#[derive(StableAbi)]` and `#[derive(GetStaticEquivalent)]`
</b>

*/
#[doc(hidden)]
#[proc_macro]
#[allow(non_snake_case)]
pub fn impl_InterfaceType(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( input, impl_interfacetype::the_macro ).into()
}





/**

This attribute is used for functions which export a module in an `implementation crate`.

When applied it creates a mangled function which calls the annotated function,
as well as check its type signature.

This is applied to functions like this:

```ignore

use abi_stable::prefix_type::PrefixTypeTrait;

#[export_root_module]
pub fn get_hello_world_mod() -> &'static TextOperationsMod {
    TextOperationsModVal{
        reverse_string,
    }.leak_into_prefix()
}

# fn main(){}

```

# Generated code

Exporting the root module creates a 
`static THE_NAME_USED_FOR_ALL_ROOT_MODULES:LibHeader= ... ;` 
with these things:

- The abi_stable version number used by the dynamic library.

- A constant describing the layout of the exported root module,and every type it references.

- A lazily initialized reference to the root module.

- The constructor function of the root module.

The name used for root modules is the one returned by 
`abi_stable::library::mangled_root_module_loader_name`.
Because there can't be multiple root modules for a library,
that function returns a constant.


# Remove type layout constant

One can avoid generating the type layout constant for the exported root module by using the
`#[unsafe_no_layout_constant]` attribute,
with the downside that if the layout changes(in an incompatible way)
it could be Undefined Behavior.

This attribute is useful if one wants to minimize the size of the dynamic library when 
doing a public release.

It is strongly encouraged that this attribute is used conditionally,
disabling it in Continuous Integration so that the 
binary compatibility of a dynamic library is checked at some point before releasing it.


# More examples

For a more detailed example look in the README in the repository for this crate.



*/
#[proc_macro_attribute]
pub fn export_root_module(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    crate::mangle_library_getter::mangle_library_getter_attr(attr,item)
}

/**
This macro is documented in abi_stable::docs::sabi_extern_fn
*/
#[proc_macro_attribute]
pub fn sabi_extern_fn(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    crate::sabi_extern_fn_impl::sabi_extern_fn(attr,item)
}


/**
This macro is documented in `abi_stable::docs::sabi_trait_attribute` .
*/
#[proc_macro_attribute]
pub fn sabi_trait(_attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( item, sabi_trait::derive_sabi_trait ).into()
}


#[doc(hidden)]
#[proc_macro]
pub fn concatenated_and_ranges( input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( input, concat_and_ranges::macro_impl ).into()
}


/**
This macro is documented in `abi_stable::docs::get_static_equivalent`
*/
#[proc_macro_derive(GetStaticEquivalent, attributes(sabi))]
pub fn derive_get_static_equivalent(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( input, get_static_equivalent::derive ).into()
}



#[doc(hidden)]
#[proc_macro]
pub fn get_string_length(input: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(input,|lit:syn::LitStr|{
        let len=lit.value().len();
        Ok(quote!( pub(super) const LEN:usize=#len; ))
    }).into()
}


#[doc(hidden)]
#[proc_macro]
pub fn construct_abi_header(_: TokenStream1) -> TokenStream1 {
    let abi_major=env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap();
    let abi_minor=env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap();
    quote!(
        pub const ABI_HEADER:AbiHeader=AbiHeader{
            magic_string:*b"abi stable library for Rust     ",
            abi_major:#abi_major,
            abi_minor:#abi_minor,
            _priv:(),
        };
    ).into()
}



///////////////////////////////////////////////////////////////////////////////



#[macro_use]
mod macros;

#[macro_use]
mod utils;

mod arenas;
mod attribute_parsing;
mod concat_and_ranges;
mod common_tokens;
mod composite_collections;
mod constants;
mod datastructure;
mod fn_pointer_extractor;
mod gen_params_in;
mod get_static_equivalent;
mod ignored_wrapper;
mod impl_interfacetype;
mod lifetimes;
mod literals_constructors;
mod mangle_library_getter;
mod my_visibility;
mod parse_utils;
mod sabi_extern_fn_impl;
mod set_span_visitor;
mod to_token_fn;
mod workaround;

#[cfg(test)]
mod input_code_range_tests;

#[cfg(test)]
mod test_framework;

#[doc(hidden)]
pub(crate) mod stable_abi;

#[doc(hidden)]
pub(crate) mod sabi_trait;



use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;

use syn::{DeriveInput,ItemFn};

use quote::{quote, ToTokens, quote_spanned};

#[allow(unused_imports)]
use core_extensions::prelude::*;

#[allow(unused_imports)]
use crate::{
    arenas::{AllocMethods, Arenas},
    utils::PrintDurationOnDrop,
    to_token_fn::ToTokenFnMut,
};


#[cfg(test)]
pub(crate) fn derive_stable_abi_from_str(s: &str) -> Result<TokenStream2,syn::Error> {
    syn::parse_str(s)
        .and_then(stable_abi::derive)
}

#[cfg(test)]
pub(crate) fn derive_sabi_trait_str(item: &str) -> Result<TokenStream2,syn::Error> {
    syn::parse_str(item)
        .and_then(sabi_trait::derive_sabi_trait)
}


////////////////////////////////////////////////////////////////////////////////


fn parse_or_compile_err<P,F>(input:TokenStream1,f:F)->TokenStream2
where 
    P:syn::parse::Parse,
    F:FnOnce(P)->Result<TokenStream2,syn::Error>
{
    syn::parse::<P>(input)
        .and_then(f)
        .unwrap_or_else(|e| e.to_compile_error() )
}
