extern crate proc_macro;

use {
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    syn::parse::{Error, Parse, ParseStream, Result},
    syn::punctuated::Punctuated,
    syn::spanned::Spanned,
    syn::token,
    syn::{
        braced, parenthesized, parse_macro_input, Attribute as SynAttribute, Ident, Lit, LitInt,
        Token, Visibility,
    },
};

#[proc_macro]
pub fn define_style(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro_error::entry_point(|| {
        let item = parse_macro_input!(input as StyleList);
        quote!(#item).into()
    })
}

enum Selector {
    Element(Ident),
    State(Ident),
}

impl Parse for Selector {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match &ident.to_string()[..] {
            "element" => {
                input.parse::<Token![:]>()?;
                Ok(Selector::Element(input.parse()?))
            }
            "state" => {
                input.parse::<Token![:]>()?;
                Ok(Selector::State(input.parse()?))
            }
            _ => Err(Error::new(ident.span(), "Expected a valid selector")),
        }
    }
}

impl ToTokens for Selector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Selector::Element(ident) => {
                quote!(node.type_id() == ::std::any::TypeId::of::<::moxie_native::dom::#ident>())
            }
            Selector::State(ident) => quote!(node.has_state(stringify!(#ident))),
        })
    }
}

#[derive(Clone)]
enum LengthItem {
    Pixels(f32),
    Ems(f32),
    ViewWidth(f32),
    ViewHeight(f32),
}

impl Parse for LengthItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let value = match input.parse::<Lit>()? {
            Lit::Int(int) => int.base10_parse::<f32>()?,
            Lit::Float(float) => float.base10_parse::<f32>()?,
            _ => unimplemented!(),
        };
        let ident = input.parse::<Ident>()?;
        match &ident.to_string()[..] {
            "px" => Ok(LengthItem::Pixels(value)),
            "em" => Ok(LengthItem::Ems(value)),
            "vw" => Ok(LengthItem::ViewWidth(value)),
            "vh" => Ok(LengthItem::ViewHeight(value)),
            _ => Err(Error::new(
                ident.span(),
                "Expected one of px, em, vw, or vh",
            )),
        }
    }
}

#[derive(Clone)]
enum Length {
    Const(LengthItem),
    Add(Box<Length>, Box<Length>),
    Sub(Box<Length>, Box<Length>),
}

#[derive(Default)]
struct LengthValues {
    pixels: f32,
    ems: f32,
    view_width: f32,
    view_height: f32,
}

impl Length {
    fn parse_add(input: ParseStream) -> Result<Self> {
        let left = input.parse::<LengthItem>()?;
        if input.peek(token::Add) {
            input.parse::<Token![+]>()?;
            let right = Self::parse_add(input)?;
            Ok(Length::Add(Box::new(Length::Const(left)), Box::new(right)))
        } else if input.peek(token::Sub) {
            input.parse::<Token![-]>()?;
            let right = Self::parse_add(input)?;
            Ok(Length::Sub(Box::new(Length::Const(left)), Box::new(right)))
        } else {
            Ok(Length::Const(left))
        }
    }

    fn eval(&self) -> LengthValues {
        match self {
            Length::Const(LengthItem::Pixels(value)) => LengthValues {
                pixels: *value,
                ..Default::default()
            },
            Length::Const(LengthItem::Ems(value)) => LengthValues {
                ems: *value,
                ..Default::default()
            },
            Length::Const(LengthItem::ViewWidth(value)) => LengthValues {
                view_width: *value / 100.0,
                ..Default::default()
            },
            Length::Const(LengthItem::ViewHeight(value)) => LengthValues {
                view_height: *value / 100.0,
                ..Default::default()
            },
            Length::Add(left, right) => {
                let left = left.eval();
                let right = right.eval();
                LengthValues {
                    pixels: left.pixels + right.pixels,
                    ems: left.ems + right.ems,
                    view_width: left.view_width + right.view_width,
                    view_height: left.view_height + right.view_height,
                }
            }
            Length::Sub(left, right) => {
                let left = left.eval();
                let right = right.eval();
                LengthValues {
                    pixels: left.pixels - right.pixels,
                    ems: left.ems - right.ems,
                    view_width: left.view_width - right.view_width,
                    view_height: left.view_height - right.view_height,
                }
            }
        }
    }
}

impl Parse for Length {
    fn parse(input: ParseStream) -> Result<Self> {
        Length::parse_add(input)
    }
}

impl ToTokens for Length {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LengthValues {
            pixels,
            ems,
            view_width,
            view_height,
        } = self.eval();
        tokens.extend(quote!(::moxie_native::style::Value {
            pixels: #pixels,
            ems: #ems,
            view_width: #view_width,
            view_height: #view_height,
        }));
    }
}

struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Parse for Color {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty = input.parse::<Ident>()?;
        match &ty.to_string()[..] {
            "rgb" => {
                let content;
                parenthesized!(content in input);
                let punctuated = content.parse_terminated::<LitInt, Token![,]>(LitInt::parse)?;
                if punctuated.len() != 3 {
                    return Err(Error::new(
                        punctuated
                            .last()
                            .map(|int| int.span())
                            .unwrap_or(punctuated.span()),
                        "rgb() requires exactly 3 arguments",
                    ));
                }
                let red = punctuated[0].base10_parse::<u8>()?;
                let green = punctuated[1].base10_parse::<u8>()?;
                let blue = punctuated[2].base10_parse::<u8>()?;

                Ok(Color {
                    red,
                    green,
                    blue,
                    alpha: 255,
                })
            }
            "rgba" => {
                let content;
                parenthesized!(content in input);
                let punctuated = content.parse_terminated::<LitInt, Token![,]>(LitInt::parse)?;
                if punctuated.len() != 4 {
                    return Err(Error::new(
                        punctuated
                            .last()
                            .map(|int| int.span())
                            .unwrap_or(punctuated.span()),
                        "rgba() requires exactly 4 arguments",
                    ));
                }
                let red = punctuated[0].base10_parse::<u8>()?;
                let green = punctuated[1].base10_parse::<u8>()?;
                let blue = punctuated[2].base10_parse::<u8>()?;
                let alpha = punctuated[3].base10_parse::<u8>()?;

                Ok(Color {
                    red,
                    green,
                    blue,
                    alpha,
                })
            }
            _ => return Err(Error::new(ty.span(), "Expected rgb or rgba")),
        }
    }
}

impl ToTokens for Color {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Color {
            red,
            green,
            blue,
            alpha,
        } = self;
        tokens.extend(quote!(::moxie_native::Color {
            red: #red,
            green: #green,
            blue: #blue,
            alpha: #alpha,
        }))
    }
}

fn parse_length_or_auto(input: ParseStream) -> Result<Option<Length>> {
    if let Ok(ident) = input.fork().parse::<Ident>() {
        if ident.to_string() == "auto" {
            input.parse::<Ident>()?;
            return Ok(None);
        }
    }
    Ok(Some(input.parse()?))
}

struct SideOffsets {
    left: Option<Length>,
    right: Option<Length>,
    top: Option<Length>,
    bottom: Option<Length>,
}

impl Parse for SideOffsets {
    fn parse(input: ParseStream) -> Result<Self> {
        let left: Option<Length> = input.call(parse_length_or_auto)?;
        if input.peek(Token![,]) {
            return Ok(SideOffsets {
                left: left.clone(),
                right: left.clone(),
                top: left.clone(),
                bottom: left,
            });
        }
        let top = input.call(parse_length_or_auto)?;
        let right = input.call(parse_length_or_auto)?;
        let bottom = input.call(parse_length_or_auto)?;
        Ok(SideOffsets {
            left,
            right,
            top,
            bottom,
        })
    }
}

struct OptionLength(Option<Length>);

impl ToTokens for OptionLength {
    fn to_tokens(&self, stream: &mut TokenStream) {
        if let Some(ref length) = self.0 {
            stream.extend(quote!(Some(#length)))
        } else {
            stream.extend(quote!(None))
        }
    }
}

impl ToTokens for SideOffsets {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let SideOffsets {
            left,
            right,
            top,
            bottom,
        } = self;
        let left = OptionLength(left.clone());
        let right = OptionLength(right.clone());
        let top = OptionLength(top.clone());
        let bottom = OptionLength(bottom.clone());
        stream.extend(quote!(
            ::moxie_native::style::SideOffsets {
                left: #left,
                right: #right,
                top: #top,
                bottom: #bottom,
            }
        ))
    }
}

enum Value {
    Length(Length),
    Color(Color),
    SideOffsets(SideOffsets),
    Enum(Ident, Ident),
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Value::Length(value) => tokens.extend(quote!(Some(#value))),
            Value::Color(value) => tokens.extend(quote!(Some(#value))),
            Value::SideOffsets(value) => value.to_tokens(tokens),
            Value::Enum(enum_ty, variant) => {
                tokens.extend(quote!(Some(::moxie_native::style::#enum_ty::#variant)))
            }
        }
    }
}

struct Attribute {
    name: Ident,
    value: Value,
}

struct EnumItem {
    short_name: &'static str,
    canonical_name: &'static str,
}

struct Enum {
    name: &'static str,
    variants: &'static [EnumItem],
}

impl Enum {
    fn lookup(&self, name: &str) -> Option<&'static str> {
        for item in self.variants {
            if item.short_name == name {
                return Some(item.canonical_name);
            }
        }
        None
    }
}

enum AttributeType {
    Length,
    Color,
    SideOffsets,
    Enum(Enum),
    Unknown,
}

impl AttributeType {
    fn from_name(name: &str) -> AttributeType {
        match name {
            "padding" | "margin" => AttributeType::SideOffsets,
            "width" | "height" | "text_size" => AttributeType::Length,
            "text_color" | "background_color" => AttributeType::Color,
            "direction" => AttributeType::Enum(Enum {
                name: "Direction",
                variants: &[
                    EnumItem {
                        short_name: "horizontal",
                        canonical_name: "Horizontal",
                    },
                    EnumItem {
                        short_name: "vertical",
                        canonical_name: "Vertical",
                    },
                ],
            }),
            "display" => AttributeType::Enum(Enum {
                name: "Display",
                variants: &[
                    EnumItem {
                        short_name: "block",
                        canonical_name: "Block",
                    },
                    EnumItem {
                        short_name: "inline",
                        canonical_name: "Inline",
                    },
                ],
            }),
            _ => AttributeType::Unknown,
        }
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let value = match AttributeType::from_name(name.to_string().as_ref()) {
            AttributeType::Length => Value::Length(input.parse()?),
            AttributeType::Color => Value::Color(input.parse()?),
            AttributeType::SideOffsets => Value::SideOffsets(input.parse()?),
            AttributeType::Enum(enum_ty) => {
                let ident = input.parse::<Ident>()?;
                if let Some(canonical) = enum_ty.lookup(&ident.to_string()[..]) {
                    Value::Enum(
                        Ident::new(enum_ty.name, ident.span()),
                        Ident::new(canonical, ident.span()),
                    )
                } else {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "Expected one of {}",
                            enum_ty
                                .variants
                                .iter()
                                .map(|v| v.short_name)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    ));
                }
            }
            AttributeType::Unknown => return Err(Error::new(name.span(), "Unknown attribute")),
        };
        Ok(Attribute { name, value })
    }
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let value = &self.value;
        tokens.extend(quote!(#name : #value));
    }
}

struct SubStyle {
    selectors: Vec<Selector>,
    attributes: Punctuated<Attribute, Token![,]>,
}

impl Parse for SubStyle {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![if]>()?;
        let mut selectors = vec![];
        loop {
            selectors.push(input.parse::<Selector>()?);
            if input.peek(token::Brace) {
                break;
            }
        }
        let content;
        braced!(content in input);
        let attributes = content.parse_terminated(Attribute::parse)?;
        Ok(SubStyle {
            selectors,
            attributes,
        })
    }
}

impl ToTokens for SubStyle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let selectors = &self.selectors;
        let attributes = self.attributes.iter().collect::<Vec<_>>();
        tokens.extend(quote!(
            ::moxie_native::style::SubStyle {
                selector: |node: ::moxie_native::dom::node::NodeRef| -> bool {
                    #(#selectors)&&*
                },
                attributes: ::moxie_native::style::CommonAttributes {
                    #(#attributes),*,
                    .. ::moxie_native::style::DEFAULT_ATTRIBUTES
                }
            }
        ))
    }
}

struct Style {
    outer: Vec<SynAttribute>,
    visibility: Visibility,
    name: Ident,
    attributes: Vec<Attribute>,
    sub_styles: Vec<SubStyle>,
}

impl Parse for Style {
    fn parse(input: ParseStream) -> Result<Self> {
        let outer = input.call(SynAttribute::parse_outer)?;
        let visibility = input.parse::<Visibility>()?;
        input.parse::<Token![static]>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let content;
        braced!(content in input);
        let mut attributes = vec![];
        loop {
            if content.peek(token::If) || content.is_empty() {
                break;
            }
            attributes.push(content.parse()?);
            if !content.peek(token::Comma) {
                break;
            }
            content.parse::<Token![,]>()?;
        }
        let mut sub_styles = vec![];
        while content.peek(token::If) {
            sub_styles.push(content.parse()?);
        }
        input.parse::<Token![;]>()?;
        Ok(Style {
            outer,
            visibility,
            name,
            attributes,
            sub_styles,
        })
    }
}

impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attributes = self.attributes.iter().collect::<Vec<_>>();
        let sub_styles = &self.sub_styles;
        let name = &self.name;
        let outer = &self.outer;
        let visibility = &self.visibility;
        tokens.extend(quote!(
            #(#outer)*
            #visibility static #name: ::moxie_native::style::Style = ::moxie_native::style::Style(
                &::moxie_native::style::StyleData {
                    name: stringify!(#name),
                    file: ::std::file!(),
                    line: ::std::line!(),
                    attributes: ::moxie_native::style::CommonAttributes {
                        #(#attributes),*,
                        .. ::moxie_native::style::DEFAULT_ATTRIBUTES
                    },
                    sub_styles: &[
                        #(#sub_styles),*
                    ],
                }
            );
        ));
    }
}

struct StyleList {
    styles: Vec<Style>,
}

impl Parse for StyleList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut styles = vec![];
        while !input.is_empty() {
            styles.push(input.parse()?);
        }
        Ok(StyleList { styles })
    }
}

impl ToTokens for StyleList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for style in &self.styles {
            tokens.extend(quote!(#style));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
