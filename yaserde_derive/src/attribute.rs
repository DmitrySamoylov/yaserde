use proc_macro2::token_stream::IntoIter;
use proc_macro2::Delimiter;
use proc_macro2::TokenTree;
use std::collections::BTreeMap;
use syn::Attribute;

#[derive(Debug, PartialEq, Clone)]
pub struct YaSerdeAttribute {
  pub attribute: bool,
  pub default: Option<String>,
  pub default_namespace: Option<String>,
  pub flatten: bool,
  pub namespaces: BTreeMap<String, String>,
  pub prefix: Option<String>,
  pub root: Option<String>,
  pub rename: Option<String>,
  pub skip_serializing_if: Option<String>,
  pub text: bool,
}

fn get_value(iter: &mut IntoIter) -> Option<String> {
  if let (Some(TokenTree::Punct(operator)), Some(TokenTree::Literal(value))) =
    (iter.next(), iter.next())
  {
    if operator.as_char() == '=' {
      Some(value.to_string().replace("\"", ""))
    } else {
      None
    }
  } else {
    None
  }
}

impl YaSerdeAttribute {
  pub fn parse(attrs: &[Attribute]) -> YaSerdeAttribute {
    let mut attribute = false;
    let mut flatten = false;
    let mut default = None;
    let mut default_namespace = None;
    let mut namespaces = BTreeMap::new();
    let mut prefix = None;
    let mut rename = None;
    let mut root = None;
    let mut skip_serializing_if = None;
    let mut text = false;

    for attr in attrs.iter() {
      let mut attr_iter = attr.clone().tokens.into_iter();
      if let Some(token) = attr_iter.next() {
        if let TokenTree::Group(group) = token {
          if group.delimiter() == Delimiter::Parenthesis {
            let mut attr_iter = group.stream().into_iter();

            while let Some(item) = attr_iter.next() {
              if let TokenTree::Ident(ident) = item {
                match ident.to_string().as_str() {
                  "attribute" => {
                    attribute = true;
                  }
                  "default" => {
                    default = get_value(&mut attr_iter);
                  }
                  "default_namespace" => {
                    default_namespace = get_value(&mut attr_iter);
                  }
                  "flatten" => {
                    flatten = true;
                  }
                  "namespace" => {
                    if let Some(namespace) = get_value(&mut attr_iter) {
                      let splitted: Vec<&str> = namespace.split(": ").collect();
                      if splitted.len() == 2 {
                        namespaces.insert(splitted[0].to_owned(), splitted[1].to_owned());
                      }
                      if splitted.len() == 1 {
                        namespaces.insert("".to_owned(), splitted[0].to_owned());
                      }
                    }
                  }
                  "prefix" => {
                    prefix = get_value(&mut attr_iter);
                  }
                  "rename" => {
                    rename = get_value(&mut attr_iter);
                  }
                  "root" => {
                    root = get_value(&mut attr_iter);
                  }
                  "skip_serializing_if" => {
                    skip_serializing_if = get_value(&mut attr_iter);
                  }
                  "text" => {
                    text = true;
                  }
                  _ => {}
                }
              }
            }
          }
        }
      }
    }

    YaSerdeAttribute {
      attribute,
      default,
      default_namespace,
      flatten,
      namespaces,
      prefix,
      rename,
      root,
      skip_serializing_if,
      text,
    }
  }
}

#[test]
fn parse_empty_attributes() {
  let attributes = vec![];
  let attrs = YaSerdeAttribute::parse(&attributes);

  assert_eq!(
    YaSerdeAttribute {
      attribute: false,
      default: None,
      default_namespace: None,
      flatten: false,
      namespaces: BTreeMap::new(),
      prefix: None,
      root: None,
      rename: None,
      skip_serializing_if: None,
      text: false,
    },
    attrs
  );
}

#[test]
fn parse_attributes() {
  use proc_macro2::{Span, TokenStream};
  use std::str::FromStr;
  use syn::punctuated::Punctuated;
  use syn::token::Bracket;
  use syn::token::Pound;
  use syn::AttrStyle::Outer;
  use syn::{Ident, Path, PathArguments, PathSegment};

  let mut punctuated = Punctuated::new();
  punctuated.push(PathSegment {
    ident: Ident::new("yaserde", Span::call_site()),
    arguments: PathArguments::None,
  });

  let attributes = vec![Attribute {
    pound_token: Pound {
      spans: [Span::call_site()],
    },
    style: Outer,
    bracket_token: Bracket {
      span: Span::call_site(),
    },
    path: Path {
      leading_colon: None,
      segments: punctuated,
    },
    tokens: TokenStream::from_str("(attribute)").unwrap(),
  }];

  let attrs = YaSerdeAttribute::parse(&attributes);

  assert_eq!(
    YaSerdeAttribute {
      attribute: true,
      default: None,
      default_namespace: None,
      flatten: false,
      namespaces: BTreeMap::new(),
      prefix: None,
      root: None,
      rename: None,
      skip_serializing_if: None,
      text: false,
    },
    attrs
  );
}

#[test]
fn parse_attributes_with_values() {
  use proc_macro2::{Span, TokenStream};
  use std::str::FromStr;
  use syn::punctuated::Punctuated;
  use syn::token::Bracket;
  use syn::token::Pound;
  use syn::AttrStyle::Outer;
  use syn::{Ident, Path, PathArguments, PathSegment};

  let mut punctuated = Punctuated::new();
  punctuated.push(PathSegment {
    ident: Ident::new("yaserde", Span::call_site()),
    arguments: PathArguments::None,
  });

  // #[()]
  let attributes = vec![Attribute {
    pound_token: Pound {
      spans: [Span::call_site()],
    },
    style: Outer,
    bracket_token: Bracket {
      span: Span::call_site(),
    },
    path: Path {
      leading_colon: None,
      segments: punctuated,
    },
    tokens: TokenStream::from_str("(attribute, flatten, default_namespace=\"example\", namespace=\"example: http://example.org\")").unwrap(),
  }];

  let attrs = YaSerdeAttribute::parse(&attributes);

  let mut namespaces = BTreeMap::new();
  namespaces.insert("example".to_string(), "http://example.org".to_string());

  assert_eq!(
    YaSerdeAttribute {
      attribute: true,
      default: None,
      default_namespace: Some("example".to_string()),
      flatten: true,
      namespaces,
      prefix: None,
      root: None,
      rename: None,
      skip_serializing_if: None,
      text: false,
    },
    attrs
  );
}
