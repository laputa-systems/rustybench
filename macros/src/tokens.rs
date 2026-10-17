use proc_macro::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};
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
    let mut angles = 0usize;

    for (index, token) in tokens.iter().enumerate() {
        if opens_generic(tokens, index) {
            angles += 1;
        } else if angles > 0 && closes_generic(tokens, index) {
            angles -= 1;
        } else if angles == 0 && is_punct(token, delimiter) {
            result.push(current);
            current = Vec::new();
            continue;
        }
        current.push(token.clone());
    }

    result.push(current);
    result
}

fn punct(token: &TokenTree) -> Option<&Punct> {
    match token {
        TokenTree::Punct(punct) => Some(punct),
        _ => None,
    }
}

fn opens_generic(tokens: &[TokenTree], index: usize) -> bool {
    let Some(token) = punct(&tokens[index]) else {
        return false;
    };
    if token.as_char() != '<' || token.spacing() != Spacing::Alone {
        return false;
    }
    let Some(previous) = index.checked_sub(1) else {
        return false;
    };
    match &tokens[previous] {
        TokenTree::Ident(_) => true,
        TokenTree::Punct(prev) if prev.as_char() == '>' => closes_generic(tokens, previous),
        TokenTree::Punct(prev) if prev.as_char() == ':' => matches!(
            previous.checked_sub(1).and_then(|before| punct(&tokens[before])),
            Some(before) if before.as_char() == ':'
        ),
        _ => false,
    }
}

fn closes_generic(tokens: &[TokenTree], index: usize) -> bool {
    let Some(token) = punct(&tokens[index]) else {
        return false;
    };
    if token.as_char() != '>' {
        return false;
    }
    let continues_operator = index
        .checked_sub(1)
        .and_then(|prev| punct(&tokens[prev]))
        .is_some_and(|prev| {
            prev.spacing() == Spacing::Joint && matches!(prev.as_char(), '-' | '=')
        });
    let comparison = token.spacing() == Spacing::Joint
        && matches!(tokens.get(index + 1).and_then(punct), Some(next) if next.as_char() == '=');
    !continues_operator && !comparison
}

pub fn token_stream(tokens: impl IntoIterator<Item = TokenTree>) -> TokenStream {
    tokens.into_iter().collect()
}
