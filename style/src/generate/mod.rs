use super::parse::{Attribute, Selector, Style, SubStyle};
use proc_macro2::Span;
use syn::{spanned::Spanned, Expr, ExprStruct, ItemStatic};

mod eval;
pub use eval::*;

fn generate_selector(selector: Selector) -> Expr {
    match selector {
        Selector::Element(selector) => {
            let span = selector.span();
            let ident = selector.ident;
            parse_quote_spanned!(
                span => node.is_type::<::moxie_native::dom::#ident>()
            )
        }
        Selector::State(selector) => {
            let span = selector.span();
            let ident = selector.ident;
            parse_quote_spanned!(span => node.has_state(style_impl::state::#ident()))
        }
    }
}

fn generate_attribute(attribute: Attribute) -> Expr {
    let span = attribute.span();
    let name = &attribute.name;
    let value = generate_expr(name, attribute.value);
    parse_quote_spanned!(
        span => style_impl::apply(values, style_impl::attribute::#name(), #value)
    )
}

fn generate_attributes(span: Span, attributes: impl Iterator<Item = Attribute>) -> ExprStruct {
    let attributes = attributes.map(generate_attribute);
    parse_quote_spanned!(
        span => ::moxie_native::style::Attributes {
            apply: |values: &mut ::moxie_native::style::ComputedValues| {
                #(#attributes;)*
            },
            get_attributes: || {
                vec![]
            },
        }
    )
}

fn generate_sub_style(style: SubStyle) -> ExprStruct {
    let span = style.span();
    let selectors = style.selectors.into_iter().map(generate_selector);
    let attributes = generate_attributes(style.brace.span, style.attributes.into_iter());
    parse_quote_spanned!(
        span =>
        ::moxie_native::style::SubStyle {
            selector: |node: &dyn ::moxie_native::style::NodeSelect| -> bool {
                #(#selectors)&&*
            },
            attributes: #attributes,
        }
    )
}

pub fn generate_style(style: Style) -> ItemStatic {
    let Style {
        ref outer,
        ref visibility,
        ref kw_static,
        ref name,
        ref equals,
        ref semicolon,
        ..
    } = style;
    let span = style.span();
    let attributes = generate_attributes(style.brace.span, style.attributes.into_iter());
    let sub_styles = style.sub_styles.into_iter().map(generate_sub_style);
    parse_quote_spanned!(
        span =>
        #(#outer)*
        #visibility #kw_static #name: ::moxie_native::style::Style #equals ::moxie_native::style::Style(
            &::moxie_native::style::StyleData {
                name: stringify!(#name),
                file: ::std::file!(),
                line: ::std::line!(),
                attributes: #attributes,
                sub_styles: &[
                    #(#sub_styles),*
                ],
            }
        ) #semicolon
    )
}
