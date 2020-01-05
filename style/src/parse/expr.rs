use super::Attribute;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, {braced, parenthesized, Ident, LitFloat, LitInt, LitStr, Token},
};

mod kw {
    syn::custom_keyword!(px);
    syn::custom_keyword!(em);
    syn::custom_keyword!(vw);
    syn::custom_keyword!(vh);
    syn::custom_keyword!(inherit);
}

pub enum LengthUnit {
    Pixels(kw::px),
    Ems(kw::em),
    ViewWidth(kw::vw),
    ViewHeight(kw::vh),
}

impl ToTokens for LengthUnit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            LengthUnit::Pixels(kw) => kw.to_tokens(tokens),
            LengthUnit::Ems(kw) => kw.to_tokens(tokens),
            LengthUnit::ViewWidth(kw) => kw.to_tokens(tokens),
            LengthUnit::ViewHeight(kw) => kw.to_tokens(tokens),
        }
    }
}

pub enum Operator {
    Add(Token![+]),
    Sub(Token![-]),
}

impl ToTokens for Operator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Operator::Add(symbol) => symbol.to_tokens(tokens),
            Operator::Sub(symbol) => symbol.to_tokens(tokens),
        }
    }
}

pub struct BinaryExpr {
    pub left: Expr,
    pub oper: Operator,
    pub right: Expr,
}

impl ToTokens for BinaryExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.left.to_tokens(tokens);
        self.oper.to_tokens(tokens);
        self.right.to_tokens(tokens);
    }
}

pub struct LengthExpr {
    pub unit: LengthUnit,
    pub expr: Expr,
}

impl ToTokens for LengthExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.expr.to_tokens(tokens);
        self.unit.to_tokens(tokens);
    }
}

pub struct StructExpr {
    pub name: Ident,
    pub brace: token::Brace,
    pub fields: Punctuated<Attribute, Token![,]>,
}

impl ToTokens for StructExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
        let fields = &self.fields;
        tokens.extend(quote_spanned!(self.brace.span => { #fields }));
    }
}

pub struct CallExpr {
    pub name: Ident,
    pub paren: token::Paren,
    pub args: Punctuated<Expr, Token![,]>,
}

impl ToTokens for CallExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
        let args = &self.args;
        tokens.extend(quote_spanned!(self.paren.span => (#args)));
    }
}

pub enum Expr {
    Int(LitInt),
    Float(LitFloat),
    Text(LitStr),
    Enum(Ident),
    LengthExpr(Box<LengthExpr>),
    BinaryExpr(Box<BinaryExpr>),
    Struct(Box<StructExpr>),
    Call(Box<CallExpr>),
    Inherit(kw::inherit),
    Error(Span),
}

impl Expr {
    fn new_bin(left: Expr, right: Expr, oper: Operator) -> Expr {
        Expr::BinaryExpr(Box::new(BinaryExpr { left, oper, right }))
    }

    fn new_length(unit: LengthUnit, expr: Expr) -> Expr {
        Expr::LengthExpr(Box::new(LengthExpr { unit, expr }))
    }

    fn parse_terminal(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let expr = if lookahead.peek(kw::inherit) {
            Expr::Inherit(input.parse()?)
        } else if lookahead.peek(LitInt) {
            Expr::Int(input.parse()?)
        } else if lookahead.peek(LitFloat) {
            Expr::Float(input.parse()?)
        } else if lookahead.peek(LitStr) {
            Expr::Text(input.parse()?)
        } else if lookahead.peek(token::Paren) {
            let contents;
            parenthesized!(contents in input);
            match contents.call(Expr::parse_add) {
                Ok(expr) => expr,
                Err(err) => {
                    let span = err.span();
                    emit_error!(err);
                    Expr::Error(span)
                }
            }
        } else if lookahead.peek(Ident) {
            let ident = input.parse()?;
            if input.peek(token::Paren) {
                // call
                let name = ident;
                let contents;
                let paren = parenthesized!(contents in input);
                let args = match contents.parse_terminated::<Expr, Token![,]>(Expr::parse) {
                    Ok(punctuated) => punctuated,
                    Err(err) => {
                        let span = err.span();
                        emit_error!(span, err);
                        Punctuated::default()
                    }
                };
                Expr::Call(Box::new(CallExpr { name, paren, args }))
            } else if input.peek(token::Brace) {
                // struct
                let name = ident;
                let contents;
                let brace = braced!(contents in input);
                let fields =
                    match contents.parse_terminated::<Attribute, Token![,]>(Attribute::parse) {
                        Ok(punctuated) => punctuated,
                        Err(err) => {
                            let span = err.span();
                            emit_error!(span, err);
                            Punctuated::default()
                        }
                    };
                Expr::Struct(Box::new(StructExpr {
                    name,
                    brace,
                    fields,
                }))
            } else {
                // enum
                Expr::Enum(ident)
            }
        } else {
            let err = lookahead.error();
            let span = err.span();
            emit_error!(err);
            Expr::Error(span)
        };
        Ok(expr)
    }

    fn parse_suffix(input: ParseStream) -> Result<Self> {
        let left = input.call(Expr::parse_terminal)?;
        if input.peek(kw::px) {
            let kw = input.parse::<kw::px>()?;
            Ok(Expr::new_length(LengthUnit::Pixels(kw), left))
        } else if input.peek(kw::em) {
            let kw = input.parse::<kw::em>()?;
            Ok(Expr::new_length(LengthUnit::Ems(kw), left))
        } else if input.peek(kw::vw) {
            let kw = input.parse::<kw::vw>()?;
            Ok(Expr::new_length(LengthUnit::ViewWidth(kw), left))
        } else if input.peek(kw::vh) {
            let kw = input.parse::<kw::vh>()?;
            Ok(Expr::new_length(LengthUnit::ViewHeight(kw), left))
        } else {
            Ok(left)
        }
    }

    fn parse_add(input: ParseStream) -> Result<Self> {
        let left = input.call(Expr::parse_suffix)?;

        if input.peek(Token![+]) {
            let op = input.parse::<Token![+]>()?;
            let right = input.call(Expr::parse_suffix)?;
            Ok(Expr::new_bin(left, right, Operator::Add(op)))
        } else if input.peek(Token![-]) {
            let op = input.parse::<Token![-]>()?;
            let right = input.call(Expr::parse_suffix)?;
            Ok(Expr::new_bin(left, right, Operator::Sub(op)))
        } else {
            Ok(left)
        }
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        input.call(Expr::parse_add)
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Expr::Int(value) => value.to_tokens(tokens),
            Expr::Float(value) => value.to_tokens(tokens),
            Expr::Text(value) => value.to_tokens(tokens),
            Expr::Enum(value) => value.to_tokens(tokens),
            Expr::LengthExpr(value) => value.to_tokens(tokens),
            Expr::BinaryExpr(value) => value.to_tokens(tokens),
            Expr::Struct(value) => value.to_tokens(tokens),
            Expr::Call(value) => value.to_tokens(tokens),
            Expr::Inherit(value) => value.to_tokens(tokens),
            Expr::Error(value) => tokens.extend(quote_spanned!(*value => <syntax error>)),
        }
    }
}
