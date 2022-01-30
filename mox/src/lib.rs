extern crate proc_macro;

use {
    proc_macro2::{Delimiter, Group, Ident, Literal, TokenStream, TokenTree},
    proc_macro_error::{abort, emit_error, proc_macro_error, Diagnostic, Level, ResultExt},
    quote::{quote, ToTokens},
    snax::{ParseError, SnaxAttribute, SnaxFragment, SnaxItem, SnaxSelfClosingTag, SnaxTag},
};

#[proc_macro_error]
#[proc_macro_hack::proc_macro_hack]
pub fn mox(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = snax::parse(input.into())
        .map_err(Error::SnaxError)
        .unwrap_or_abort();
    let item = MoxItem::from(item);
    quote!(#item).into()
}

enum MoxItem {
    Tag(MoxTag),
    TagNoChildren(MoxTagNoChildren),
    Fragment(Vec<MoxItem>),
    Content(TokenTree),
}

impl From<SnaxItem> for MoxItem {
    fn from(item: SnaxItem) -> Self {
        match item {
            SnaxItem::Tag(t) => MoxItem::Tag(MoxTag::from(t)),
            SnaxItem::SelfClosingTag(t) => MoxItem::TagNoChildren(MoxTagNoChildren::from(t)),
            SnaxItem::Fragment(SnaxFragment { children }) => {
                MoxItem::Fragment(children.into_iter().map(MoxItem::from).collect())
            }
            SnaxItem::Content(atom) => MoxItem::Content(wrap_content_tokens(atom)),
        }
    }
}

fn wrap_content_tokens(tt: TokenTree) -> TokenTree {
    let mut result = Group::new(Delimiter::Brace, quote!(#tt)).into();
    match tt {
        TokenTree::Group(g) => {
            let mut tokens = g.stream().into_iter();
            if let Some(TokenTree::Punct(p)) = tokens.next() {
                if p.as_char() == '%' {
                    // strip the percent sign off the front
                    let mut args = TokenStream::new();
                    args.extend(tokens);

                    // TODO get all but the last element here too if its a %
                    result = Group::new(Delimiter::Parenthesis, quote!(format!(#args))).into();
                }
            }
        }
        tt @ TokenTree::Ident(_) | tt @ TokenTree::Literal(_) => {
            result = tt;
        }
        TokenTree::Punct(p) => emit_error!(p.span(), "'{}' not valid in item position", p),
    }
    result
}

impl ToTokens for MoxItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MoxItem::Tag(tag) => tag.to_tokens(tokens),
            MoxItem::TagNoChildren(tag) => tag.to_tokens(tokens),
            MoxItem::Fragment(children) => tokens.extend(quote!(
                mox_impl::fragment() #( .add_child(#children) )* .build()
            )),
            MoxItem::Content(content) => content.to_tokens(tokens),
        }
    }
}

struct MoxTag {
    name: Ident,
    fn_args: Option<MoxArgs>,
    attributes: Vec<MoxAttr>,
    children: Vec<MoxItem>,
}

impl ToTokens for MoxTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tag_to_tokens(
            &self.name,
            &self.fn_args,
            &self.attributes,
            Some(&self.children),
            tokens,
        );
    }
}

fn args_and_attrs(snaxttrs: Vec<SnaxAttribute>) -> (Option<MoxArgs>, Vec<MoxAttr>) {
    let mut snaxs = snaxttrs.into_iter().peekable();
    let mut args = None;

    if let Some(first) = snaxs.peek() {
        let name = match first {
            SnaxAttribute::Simple { name, .. } => name,
        };

        if name == "_" {
            let first = snaxs.next().unwrap();
            args = Some(MoxArgs {
                value: match first {
                    SnaxAttribute::Simple { value, .. } => value,
                },
            });
        }
    }

    (args, snaxs.map(MoxAttr::from).collect())
}

fn tag_to_tokens(
    name: &Ident,
    fn_args: &Option<MoxArgs>,
    attributes: &[MoxAttr],
    children: Option<&[MoxItem]>,
    stream: &mut TokenStream,
) {
    // this needs to be nested within other token groups, must be accumulated separately from stream
    let mut contents = quote!();

    for attr in attributes {
        attr.to_tokens(&mut contents);
    }

    if let Some(items) = children {
        for item in items {
            let ts = item.to_token_stream();
            contents.extend(quote!(.add_child(#ts)));
        }
    }

    let fn_args = fn_args.as_ref().map(|args| match &args.value {
        TokenTree::Group(g) => {
            // strip trailing commas that would bork macro parsing
            let mut tokens: Vec<TokenTree> = g.stream().into_iter().collect();

            let last = tokens
                .get(tokens.len() - 1)
                .expect("function argument delimiters must contain some tokens");

            if last.to_string() == "," {
                tokens.truncate(tokens.len() - 1);
            }

            let mut without_delim = TokenStream::new();
            without_delim.extend(tokens);
            quote!(#without_delim)
        }
        _ => unimplemented!("bare function args (without a paired delimiter) aren't supported yet"),
    });

    let invocation = if contents.is_empty() {
        quote!(#name(#fn_args))
    } else {
        if fn_args.is_some() {
            unimplemented!(
                "can't emit function arguments at the same time as attributes or children yet"
            )
        }
        quote!(mox_impl::elt::#name() #contents .build())
    };

    stream.extend(invocation);
}

impl From<SnaxTag> for MoxTag {
    fn from(
        SnaxTag {
            name,
            attributes,
            children,
        }: SnaxTag,
    ) -> Self {
        let (fn_args, attributes) = args_and_attrs(attributes);
        Self {
            name,
            fn_args,
            attributes,
            children: children.into_iter().map(MoxItem::from).collect(),
        }
    }
}

struct MoxTagNoChildren {
    name: Ident,
    fn_args: Option<MoxArgs>,
    attributes: Vec<MoxAttr>,
}

impl ToTokens for MoxTagNoChildren {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tag_to_tokens(&self.name, &self.fn_args, &self.attributes, None, tokens);
    }
}

impl From<SnaxSelfClosingTag> for MoxTagNoChildren {
    fn from(SnaxSelfClosingTag { name, attributes }: SnaxSelfClosingTag) -> Self {
        let (fn_args, attributes) = args_and_attrs(attributes);
        Self {
            name,
            fn_args,
            attributes,
        }
    }
}

struct MoxArgs {
    value: TokenTree,
}

enum MoxAttr {
    Simple { name: Ident, value: TokenTree },
    Handler { name: Ident, value: TokenTree },
    Data { name: Ident, value: TokenTree },
}

impl ToTokens for MoxAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match self {
            MoxAttr::Simple { name, value } => quote!(.set_attr(mox_impl::attr::#name(), #value)),
            MoxAttr::Handler { name, value } => quote!(.on_event(mox_impl::event::#name(), #value)),
            MoxAttr::Data { name, value } => {
                let mut literal = Literal::string(name.to_string().as_ref());
                literal.set_span(name.span());
                quote!(.set_data(#literal, #value))
            }
        };

        tokens.extend(stream);
    }
}

impl From<SnaxAttribute> for MoxAttr {
    fn from(attr: SnaxAttribute) -> Self {
        match attr {
            SnaxAttribute::Simple { name, value } => {
                let name_str = name.to_string();
                if name_str == "_" {
                    abort!(
                        name.span(),
                        "anonymous attributes are only allowed in the first position"
                    );
                } else if name_str.starts_with("data_") {
                    MoxAttr::Data { name, value }
                } else if name_str.starts_with("on_") {
                    MoxAttr::Handler { name, value }
                } else {
                    MoxAttr::Simple { name, value }
                }
            }
        }
    }
}

enum Error {
    SnaxError(ParseError),
}

impl Into<Diagnostic> for Error {
    fn into(self) -> Diagnostic {
        match self {
            Error::SnaxError(ParseError::UnexpectedEnd) => {
                Diagnostic::new(Level::Error, format!("input ends before expected"))
            }
            Error::SnaxError(ParseError::UnexpectedItem(item)) => {
                // TODO https://github.com/LPGhatguy/snax/issues/9
                Diagnostic::new(Level::Error, format!("did not expect {:?}", item))
            }
            Error::SnaxError(ParseError::UnexpectedToken(token)) => Diagnostic::spanned(
                token.span(),
                Level::Error,
                format!("did not expect '{}'", token),
            ),
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
