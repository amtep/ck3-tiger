extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod verify;

#[proc_macro_derive(Verify)]
#[proc_macro_error]
pub fn derive_mod_verify(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    verify::derive_verify(input).into()
}
