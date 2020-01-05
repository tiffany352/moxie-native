use crate::parse::{Expr, LengthUnit};
use quote::quote_spanned;
use syn::{spanned::Spanned, Expr as SynExpr, Ident, LitFloat};

pub fn generate_expr(attr_name: &Ident, expr: Expr) -> SynExpr {
    match expr {
        Expr::Int(int) => {
            let value_str = format!("{}f64", int.base10_digits());
            let value = LitFloat::new(&value_str[..], int.span());
            parse_quote_spanned!(
                int.span() => #value
            )
        }
        Expr::Float(float) => {
            let value_str = format!("{}f64", float.base10_digits());
            let value = LitFloat::new(&value_str[..], float.span());
            parse_quote_spanned!(
                float.span() => #value
            )
        }
        Expr::Text(text) => parse_quote_spanned!(text.span() => #text),
        Expr::Enum(ident) => parse_quote_spanned!(
            ident.span() => style_impl::keyword::#ident()
        ),
        Expr::LengthExpr(expr) => {
            let res = generate_expr(attr_name, expr.expr);
            match expr.unit {
                LengthUnit::Pixels(kw) => parse_quote_spanned!(kw.span => style_impl::pixels(#res)),
                LengthUnit::Ems(kw) => parse_quote_spanned!(kw.span => style_impl::ems(#res)),
                LengthUnit::ViewWidth(kw) => {
                    parse_quote_spanned!(kw.span => style_impl::view_width(#res))
                }
                LengthUnit::ViewHeight(kw) => {
                    parse_quote_spanned!(kw.span => style_impl::view_height(#res))
                }
            }
        }
        Expr::BinaryExpr(exp) => {
            let span = exp.span();
            let left = generate_expr(attr_name, exp.left);
            let right = generate_expr(attr_name, exp.right);
            let op = &exp.oper;
            parse_quote_spanned!(span => #left #op #right)
        }
        Expr::Struct(expr) => {
            let span = expr.span();
            let name = &expr.name;
            let fields = expr.fields.into_iter().map(|attr| {
                let span = attr.span();
                let name = &attr.name;
                let value = generate_expr(attr_name, attr.value);
                quote_spanned!(
                    span =>
                    .#name(#value)
                )
            });
            parse_quote_spanned!(
                    span =>
                    style_impl::types::#name::new()
                    #(#fields)*
                    .build()
            )
        }
        Expr::Call(expr) => {
            let span = expr.span();
            let name = &expr.name;
            let args = expr
                .args
                .into_iter()
                .map(|expr| generate_expr(attr_name, expr));
            parse_quote_spanned!(
                span => style_impl::func::#name(#(#args),*)
            )
        }
        Expr::Inherit(kw) => parse_quote_spanned!(kw.span => style_impl::Inherit),
        Expr::Error(span) => parse_quote_spanned!(span => ()),
    }
}
