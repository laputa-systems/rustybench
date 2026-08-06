use proc_macro::{Delimiter, Group, Literal, TokenStream, TokenTree};
use std::str::FromStr;

pub fn parse(source: impl AsRef<str>) -> TokenStream {
    let source = source.as_ref();
    TokenStream::from_str(source)
        .unwrap_or_else(|error| panic!("rustybench generated invalid Rust: {error}: {source}"))
}

pub fn display(tokens: &TokenStream) -> String {
    tokens.to_string()
}

pub fn string_literal(value: &str) -> String {
    Literal::string(value).to_string()
}

pub fn is_ident(token: &TokenTree, expected: &str) -> bool {
    matches!(token, TokenTree::Ident(ident) if ident.to_string() == expected)
}

pub fn is_punct(token: &TokenTree, expected: char) -> bool {
    matches!(token, TokenTree::Punct(punct) if punct.as_char() == expected)
}

pub fn group(token: &TokenTree, delimiter: Delimiter) -> Option<&Group> {
    match token {
        TokenTree::Group(group) if group.delimiter() == delimiter => Some(group),
        _ => None,
    }
}

pub fn split_top_level(tokens: &[TokenTree], delimiter: char) -> Vec<Vec<TokenTree>> {
    let mut result = Vec::new();
    let mut current = Vec::new();

    for token in tokens {
        if is_punct(token, delimiter) {
            result.push(current);
            current = Vec::new();
        } else {
            current.push(token.clone());
        }
    }

    result.push(current);
    result
}

pub fn token_stream(tokens: impl IntoIterator<Item = TokenTree>) -> TokenStream {
    tokens.into_iter().collect()
}
