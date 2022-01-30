extern crate proc_macro;

use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::parse_macro_input;

macro_rules! parse_quote_spanned {
    ($span:expr => $($tt:tt)*) => {
        ::syn::parse_quote::parse(
            std::convert::From::from(
                ::quote::quote_spanned!($span => $($tt)*)
            )
        )
    };
}

mod generate;
mod parse;

#[proc_macro_error]
#[proc_macro]
pub fn define_style(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as parse::StyleList);
    let mut tokens = proc_macro2::TokenStream::new();
    for ast in item.styles {
        let style = generate::generate_style(ast);
        tokens.extend(quote!(#style));
    }
    tokens.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
