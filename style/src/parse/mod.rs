use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
    token, {braced, Attribute as SynAttribute, Ident, Token, Visibility},
};

mod expr;

pub use expr::*;

mod kw {
    syn::custom_keyword!(element);
    syn::custom_keyword!(state);
}

pub struct ElementSelector {
    pub kw: kw::element,
    pub colon: Token![:],
    pub ident: Ident,
}

impl Parse for ElementSelector {
    fn parse(input: ParseStream) -> Result<Self> {
        let kw = input.parse::<kw::element>()?;
        let colon = input.parse::<Token![:]>()?;
        let ident = input.parse::<Ident>()?;
        Ok(ElementSelector { kw, colon, ident })
    }
}

impl ToTokens for ElementSelector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kw.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.ident.to_tokens(tokens);
    }
}

pub struct StateSelector {
    pub kw: kw::state,
    pub colon: Token![:],
    pub ident: Ident,
}

impl Parse for StateSelector {
    fn parse(input: ParseStream) -> Result<Self> {
        let kw = input.parse::<kw::state>()?;
        let colon = input.parse::<Token![:]>()?;
        let ident = input.parse::<Ident>()?;
        Ok(StateSelector { kw, colon, ident })
    }
}
impl ToTokens for StateSelector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kw.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.ident.to_tokens(tokens);
    }
}

pub enum Selector {
    Element(ElementSelector),
    State(StateSelector),
}

impl Parse for Selector {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::element) {
            Ok(Selector::Element(input.parse()?))
        } else if lookahead.peek(kw::state) {
            Ok(Selector::State(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Selector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Selector::Element(selector) => selector.to_tokens(tokens),
            Selector::State(selector) => selector.to_tokens(tokens),
        }
    }
}

pub struct Attribute {
    pub name: Ident,
    pub colon: Token![:],
    pub value: Expr,
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let colon = input.parse::<Token![:]>()?;
        let value = input.parse::<Expr>()?;
        Ok(Attribute { name, colon, value })
    }
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

pub struct SubStyle {
    pub kw: Token![if],
    pub selectors: Punctuated<Selector, Token![&&]>,
    pub brace: token::Brace,
    pub attributes: Punctuated<Attribute, Token![,]>,
}

impl Parse for SubStyle {
    fn parse(input: ParseStream) -> Result<Self> {
        let kw = input.parse::<Token![if]>()?;
        let mut selectors = Punctuated::new();
        loop {
            selectors.push_value(input.parse::<Selector>()?);
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![&&]) {
                selectors.push_punct(input.parse::<Token![&&]>()?);
            } else if lookahead.peek(token::Brace) {
                break;
            } else {
                return Err(lookahead.error());
            }
        }
        let content;
        let brace = braced!(content in input);
        let attributes = content.parse_terminated(Attribute::parse);
        let attributes = match attributes {
            Ok(attributes) => attributes,
            Err(err) => {
                emit_error!(err);
                Punctuated::default()
            }
        };
        Ok(SubStyle {
            kw,
            selectors,
            brace,
            attributes,
        })
    }
}

impl ToTokens for SubStyle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kw.to_tokens(tokens);
        let selectors = &self.selectors;
        tokens.extend(quote_spanned!(
            self.selectors.span() =>
            #(#selectors)&&+
        ));
        let attributes = self.attributes.iter();
        tokens.extend(quote_spanned!(
            self.brace.span => {
                #(#attributes),*
            }
        ))
    }
}

pub struct Style {
    pub outer: Vec<SynAttribute>,
    pub visibility: Visibility,
    pub kw_static: Token![static],
    pub name: Ident,
    pub equals: Token![=],
    pub brace: token::Brace,
    pub attributes: Vec<Attribute>,
    pub sub_styles: Vec<SubStyle>,
    pub semicolon: Token![;],
}

fn parse_style_body(input: ParseStream) -> Result<(Vec<Attribute>, Vec<SubStyle>)> {
    let mut attributes = vec![];
    loop {
        if input.peek(token::If) || input.is_empty() {
            break;
        }
        attributes.push(input.parse()?);
        if !input.peek(token::Comma) {
            break;
        }
        if input.peek(Token![;]) {
            emit_error!(input.error("Expected `,`, got `;`"));
        } else {
            input.parse::<Token![,]>()?;
        }
    }
    let mut sub_styles = vec![];
    while input.peek(token::If) {
        sub_styles.push(input.parse()?);
    }
    Ok((attributes, sub_styles))
}

impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Style {
            ref outer,
            ref visibility,
            ref kw_static,
            ref name,
            ref equals,
            ref brace,
            ref attributes,
            ref sub_styles,
            ref semicolon,
        } = self;
        let attributes = attributes.iter();
        let sub_styles = sub_styles.iter();
        tokens.extend(quote_spanned!(
            brace.span =>
            #(#outer)*
            #visibility
            #kw_static
            #name
            #equals
            #(#attributes,)*
            #(#sub_styles)*
            #semicolon
        ));
    }
}

impl Parse for Style {
    fn parse(input: ParseStream) -> Result<Self> {
        let outer = input.call(SynAttribute::parse_outer)?;
        let visibility = input.parse::<Visibility>()?;
        let kw_static = input.parse::<Token![static]>()?;
        let name = input.parse::<Ident>()?;
        let equals = input.parse::<Token![=]>()?;
        let content;
        let brace = braced!(content in input);
        let (attributes, sub_styles) = match parse_style_body(&content) {
            Ok(result) => result,
            Err(err) => {
                emit_error!(err.span(), err);
                (vec![], vec![])
            }
        };
        let semicolon = input.parse::<Token![;]>()?;
        Ok(Style {
            outer,
            visibility,
            kw_static,
            name,
            equals,
            brace,
            attributes,
            sub_styles,
            semicolon,
        })
    }
}

pub struct StyleList {
    pub styles: Vec<Style>,
}

impl Parse for StyleList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut styles = vec![];
        while !input.is_empty() {
            match input.parse() {
                Ok(style) => styles.push(style),
                Err(err) => {
                    emit_error!(err);
                    return Ok(StyleList { styles });
                }
            };
        }
        Ok(StyleList { styles })
    }
}
