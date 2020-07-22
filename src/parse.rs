use heck::SnakeCase;
use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Data, DataStruct, DataUnion, DeriveInput, Error, Field, Fields, FieldsUnnamed,
    Generics, Ident, Path, Result, Token,
};

pub struct MatcherDerive {
    pub enum_name: Ident,
    pub matcher_name: Option<Ident>,
    pub generics: Generics,
    pub variants: Vec<MatcherVariant>,
}

impl MatcherDerive {
    pub fn resolve_matcher_name(&self) -> Ident {
        self.matcher_name.as_ref().cloned().unwrap_or_else(|| {
            // TODO at least think about how this name is constructed, make it a concious choice.
            Ident::new(&format!("{}Matcher", self.enum_name), self.enum_name.span())
        })
    }
}

impl Parse for MatcherDerive {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;
        let enum_name = input.ident;
        let generics = input.generics;
        let data = match input.data {
            Data::Enum(data) => Ok(data),
            Data::Struct(DataStruct { struct_token, .. }) => enum_error(struct_token.span),
            Data::Union(DataUnion { union_token, .. }) => enum_error(union_token.span),
        }?;
        let mut matcher_name = None;
        for attr in process_attrs(input.attrs) {
            match attr? {
                MatcherAttr::BuilderName(_, span) => {
                    return Err(Error::new(span, "attribute not valid on enum"))
                }
                MatcherAttr::MatcherName(name, _) => matcher_name = Some(name),
            }
        }
        let mut variants = Vec::new();
        for variant in data.variants {
            let variant_name = variant.ident;
            let name_span = variant_name.span();
            let field = match variant.fields {
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => match iter_get_one(unnamed) {
                    Some(f) => f,
                    None => return variant_error(name_span),
                },
                _ => return variant_error(name_span),
            };
            let attrs = VariantAttrs::parse(variant.attrs)?;
            variants.push(MatcherVariant {
                builder_name: attrs.builder_name,
                name: variant_name,
                field,
            });
        }
        Ok(MatcherDerive {
            enum_name,
            matcher_name,
            generics,
            variants,
        })
    }
}

fn enum_error<T>(span: Span) -> Result<T> {
    Err(Error::new(span, "only `enum`s can implement `Matcher`"))
}

fn variant_error<T>(span: Span) -> Result<T> {
    Err(Error::new(
        span,
        "the variant's data must be a single unnamed field",
    ))
}

pub struct MatcherVariant {
    pub builder_name: Option<Ident>,
    pub name: Ident,
    pub field: Field,
}

impl MatcherVariant {
    pub fn resolve_builder_name(&self) -> Ident {
        self.builder_name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| snakify(&self.name))
    }
}

#[derive(Default)]
struct VariantAttrs {
    /// The name of the function call to build the corresponding widget.
    builder_name: Option<Ident>,
}

impl VariantAttrs {
    fn parse(input: Vec<Attribute>) -> Result<Self> {
        let mut matcher_attrs = VariantAttrs::default();
        for attr in process_attrs(input) {
            match attr? {
                MatcherAttr::BuilderName(builder_name, _) => {
                    matcher_attrs.builder_name = Some(builder_name)
                }
                MatcherAttr::MatcherName(_, span) => {
                    return Err(Error::new(span, "attribute not valid on variants"))
                }
            }
        }
        Ok(matcher_attrs)
    }
}

// spans are for error reporting.
enum MatcherAttr {
    MatcherName(Ident, Span),
    BuilderName(Ident, Span),
}

impl Parse for MatcherAttr {
    fn parse(s: ParseStream) -> Result<Self> {
        let attr_name = s.parse::<Ident>()?;
        let name_span = attr_name.span();
        match attr_name.to_string().as_str() {
            "builder_name" => {
                s.parse::<Token![=]>()?;
                s.parse()
                    .map(|builder_name| MatcherAttr::BuilderName(builder_name, name_span))
            }
            "matcher_name" => {
                s.parse::<Token![=]>()?;
                s.parse()
                    .map(|matcher_name| MatcherAttr::MatcherName(matcher_name, name_span))
            }
            other => Err(Error::new(
                name_span,
                format!("expected `builder_name`, found `{}`", other),
            )),
        }
    }
}

/// Go through a set of `Attribute`s and process them into an iterator of parsed attributes
fn process_attrs(input: Vec<Attribute>) -> impl Iterator<Item = Result<MatcherAttr>> {
    ProcessAttrs {
        attrs: input.into_iter(),
        part: None,
    }
}

struct ProcessAttrs {
    attrs: std::vec::IntoIter<Attribute>,
    part: Option<syn::punctuated::IntoIter<MatcherAttr>>,
}

impl ProcessAttrs {
    /// Get the next part, or `None` if there aren't any.
    fn next_part(&mut self) -> Option<MatcherAttr> {
        match &mut self.part {
            Some(iter) => iter.next(),
            None => None,
        }
    }

    /// Find the next `matches` attr and load it into `part`
    fn load_parts(&mut self) -> Result<()> {
        assert!(!self.part.as_mut().and_then(|iter| iter.next()).is_some());
        loop {
            let attr = match self.attrs.next() {
                Some(a) => a,
                None => return Ok(()),
            };
            if !matches_path(&attr.path) {
                continue;
            }
            let match_attrs: Punctuated<_, Token![,]> =
                attr.parse_args_with(Punctuated::parse_terminated)?;
            self.part = Some(match_attrs.into_iter());
            return Ok(());
        }
    }
}

impl Iterator for ProcessAttrs {
    type Item = Result<MatcherAttr>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(matcher_attr) = self.next_part() {
            return Some(Ok(matcher_attr));
        }
        if let Err(e) = self.load_parts() {
            return Some(Err(e));
        }
        self.next_part().map(Ok)
    }
}

/// True if the path is exactly `matches`.
fn matches_path(p: &Path) -> bool {
    if p.leading_colon.is_some() {
        return false;
    }
    let segment = match iter_get_one(p.segments.iter()) {
        Some(s) => s,
        None => return false,
    };
    segment.ident == "matcher" && segment.arguments.is_empty()
}

fn iter_get_one<T, I: IntoIterator<Item = T>>(iter: I) -> Option<T> {
    let mut iter = iter.into_iter();
    let result = iter.next()?;
    if iter.next().is_some() {
        return None;
    }
    Some(result)
}

fn snakify(input: &Ident) -> Ident {
    let new_name = input.to_string().to_snake_case();
    Ident::new(&new_name, input.span())
}
