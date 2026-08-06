use proc_macro::{Delimiter, TokenStream, TokenTree};

use crate::{FunctionInfo, MacroKind, tokens};

/// Values from parsed options shared between `#[rustybench::bench]` and
/// `#[rustybench::bench_group]`.
pub(crate) struct AttrOptions {
    /// `rustybench::__private` as source text.
    pub private_mod: String,

    /// Custom name for the benchmark or group.
    pub name_expr: Option<TokenStream>,

    /// `IntoIterator` from which to provide runtime arguments.
    pub args_expr: Option<ExprValue>,

    /// Options for generic functions.
    pub generic: GenericOptions,

    /// The `BenchOptions.counters` field and its value, followed by a comma.
    pub counters: TokenStream,

    /// Options used directly as `BenchOptions` fields.
    pub bench_options: Vec<(String, TokenStream)>,
}

impl AttrOptions {
    pub fn parse(
        input: TokenStream,
        target_macro: MacroKind,
        function: Option<&FunctionInfo>,
    ) -> Result<Self, TokenStream> {
        let macro_name = target_macro.name();
        let mut rustybench_crate = None;
        let mut name_expr = None;
        let mut args_expr = None;
        let mut bench_options = Vec::new();
        let mut counter_values = Vec::new();
        let mut counters_field = None;
        let mut generic = GenericOptions::default();
        let mut seen_convenience = [false; 4];
        let mut explicit_counter = false;

        let input: Vec<_> = input.into_iter().collect();
        for option in tokens::split_top_level(&input, ',') {
            if option.is_empty() {
                continue;
            }

            let Some(TokenTree::Ident(ident)) = option.first() else {
                return Err(error(macro_name, "unsupported option"));
            };

            let key = ident.to_string();
            let normalized = key.strip_prefix("r#").unwrap_or(&key);
            let equals = option.iter().position(|token| tokens::is_punct(token, '='));
            let value = equals
                .map(|index| tokens::token_stream(option[index + 1..].iter().cloned()))
                .unwrap_or_else(|| tokens::parse("true"));

            if equals.is_some_and(|index| index != 1) {
                return Err(error(macro_name, "unsupported option"));
            }

            match normalized {
                "crate" => set_once(&mut rustybench_crate, value, macro_name, normalized)?,
                "name" => set_once(&mut name_expr, value, macro_name, normalized)?,
                "types" => {
                    let Some(function) = function else {
                        return Err(unsupported(macro_name, normalized));
                    };
                    if !function.has_type_param {
                        return Err(error(
                            macro_name,
                            "generic type required for 'types' option",
                        ));
                    }
                    set_once(
                        &mut generic.types,
                        parse_types(value, macro_name)?,
                        macro_name,
                        normalized,
                    )?;
                }
                "consts" => {
                    let Some(function) = function else {
                        return Err(unsupported(macro_name, normalized));
                    };
                    if !function.has_const_param {
                        return Err(error(
                            macro_name,
                            "generic const required for 'consts' option",
                        ));
                    }
                    set_once(
                        &mut generic.consts,
                        ExprValue::new(value),
                        macro_name,
                        normalized,
                    )?;
                }
                "args" => {
                    let Some(function) = function else {
                        return Err(unsupported(macro_name, normalized));
                    };
                    if !matches!(function.arg_count, 1 | 2) {
                        return Err(error(
                            macro_name,
                            "function argument required for 'args' option",
                        ));
                    }
                    set_once(
                        &mut args_expr,
                        ExprValue::new(value),
                        macro_name,
                        normalized,
                    )?;
                }
                "counter" => {
                    if counters_field.is_some() {
                        return Err(repeated(macro_name, normalized));
                    }
                    explicit_counter = true;
                    counter_values.push((value, None));
                    counters_field = Some("counters".to_owned());
                }
                "counters" => {
                    if counters_field.is_some() {
                        return Err(repeated(macro_name, normalized));
                    }
                    explicit_counter = true;
                    let Some(values) = array_values(&value) else {
                        return Err(error(macro_name, "'counters' requires an array"));
                    };
                    counter_values.extend(values.into_iter().map(|value| (value, None)));
                    counters_field = Some(key);
                }
                "bytes_count" | "chars_count" | "cycles_count" | "items_count" => {
                    let index = match normalized {
                        "bytes_count" => 0,
                        "chars_count" => 1,
                        "cycles_count" => 2,
                        "items_count" => 3,
                        _ => unreachable!(),
                    };
                    if seen_convenience[index] {
                        return Err(repeated(macro_name, normalized));
                    }
                    seen_convenience[index] = true;
                    if explicit_counter {
                        return Err(repeated(macro_name, normalized));
                    }
                    let type_name = match normalized {
                        "bytes_count" => "BytesCount",
                        "chars_count" => "CharsCount",
                        "cycles_count" => "CyclesCount",
                        "items_count" => "ItemsCount",
                        _ => unreachable!(),
                    };
                    counter_values.push((value, Some(type_name)));
                    counters_field = Some("counters".to_owned());
                }
                _ => bench_options.push((key, value)),
            }
        }

        let private_mod = format!(
            "{}::__private",
            rustybench_crate
                .map(|value| tokens::display(&value))
                .unwrap_or_else(|| "::rustybench".to_owned())
        );

        let counters = counters_field
            .map(|field| {
                let values = counter_values
                    .into_iter()
                    .map(|(value, type_name)| match type_name {
                        Some(type_name) => format!(
                            "{{ use ::std::convert::From as _; {}::counter::{}::from({}) }}",
                            rustybench_crate_source(&private_mod),
                            type_name,
                            tokens::display(&value)
                        ),
                        None => tokens::display(&value),
                    })
                    .map(|value| format!(".with({value})"))
                    .collect::<String>();

                tokens::parse(format!(
                    "{field}: {private_mod}::new_counter_set(){values},"
                ))
            })
            .unwrap_or_else(TokenStream::new);

        Ok(Self {
            private_mod,
            name_expr,
            args_expr,
            generic,
            counters,
            bench_options,
        })
    }

    /// Produces a function expression for creating `LazyLock<BenchOptions>`.
    pub fn bench_options_fn(&self, ignored: bool) -> TokenStream {
        if self.bench_options.is_empty() && self.counters.is_empty() && !ignored {
            return tokens::parse("None");
        }

        let options = self
            .bench_options
            .iter()
            .map(|(option, value)| {
                let normalized = option.strip_prefix("r#").unwrap_or(option);
                let value = match normalized {
                    "threads" if is_literal_array(value) => {
                        format!("::std::borrow::Cow::Borrowed(&{})", tokens::display(value))
                    }
                    "threads" => format!(
                        "{}::IntoThreads::into_threads({})",
                        self.private_mod,
                        tokens::display(value)
                    ),
                    "min_time" | "max_time" => format!(
                        "{}::IntoDuration::into_duration({})",
                        self.private_mod,
                        tokens::display(value)
                    ),
                    _ => tokens::display(value),
                };
                format!("{option}: Some({value}),")
            })
            .collect::<String>();

        let ignored = if ignored { "ignore: Some(true)," } else { "" };

        format_tokens(format!(
            "Some(::std::sync::LazyLock::new(|| {{ #[allow(clippy::needless_update)] {}::BenchOptions {{ {options} {ignored} {} ..::std::default::Default::default() }} }}))",
            self.private_mod,
            tokens::display(&self.counters)
        ))
    }
}

#[derive(Default)]
pub struct GenericOptions {
    pub types: Option<Vec<TokenStream>>,
    pub consts: Option<ExprValue>,
}

impl GenericOptions {
    pub fn is_empty(&self) -> bool {
        matches!(self.types, Some(ref types) if types.is_empty())
            || matches!(self.consts, Some(ref consts) if consts.is_empty_array())
    }

    pub fn types_iter(&self) -> Box<dyn Iterator<Item = Option<&TokenStream>> + '_> {
        match &self.types {
            None => Box::new(std::iter::once(None)),
            Some(types) => Box::new(types.iter().map(Some)),
        }
    }
}

pub struct ExprValue {
    pub raw: TokenStream,
    pub array: Option<Vec<TokenStream>>,
}

impl ExprValue {
    fn new(raw: TokenStream) -> Self {
        let array = array_values(&raw);
        Self { raw, array }
    }

    pub fn is_empty_array(&self) -> bool {
        matches!(&self.array, Some(values) if values.is_empty())
    }
}

fn parse_types(value: TokenStream, macro_name: &str) -> Result<Vec<TokenStream>, TokenStream> {
    array_values(&value).ok_or_else(|| error(macro_name, "'types' requires an array"))
}

fn array_values(value: &TokenStream) -> Option<Vec<TokenStream>> {
    let tokens: Vec<_> = value.clone().into_iter().collect();
    if tokens.len() != 1 {
        return None;
    }
    let TokenTree::Group(group) = &tokens[0] else {
        return None;
    };
    if group.delimiter() != Delimiter::Bracket {
        return None;
    }
    Some(
        tokens::split_top_level(&group.stream().into_iter().collect::<Vec<_>>(), ',')
            .into_iter()
            .filter(|value| !value.is_empty())
            .map(tokens::token_stream)
            .collect(),
    )
}

fn is_literal_array(value: &TokenStream) -> bool {
    let Some(values) = array_values(value) else {
        return false;
    };
    values.iter().all(|value| {
        value
            .clone()
            .into_iter()
            .all(|token| matches!(token, TokenTree::Literal(_)))
    })
}

fn rustybench_crate_source(private_mod: &str) -> String {
    private_mod
        .strip_suffix("::__private")
        .unwrap_or(private_mod)
        .to_owned()
}

fn format_tokens(source: String) -> TokenStream {
    tokens::parse(source)
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    macro_name: &str,
    option: &str,
) -> Result<(), TokenStream> {
    if slot.is_some() {
        Err(repeated(macro_name, option))
    } else {
        *slot = Some(value);
        Ok(())
    }
}

fn repeated(macro_name: &str, option: &str) -> TokenStream {
    error(
        macro_name,
        &format!("repeated '{macro_name}' option '{option}'"),
    )
}

fn unsupported(macro_name: &str, option: &str) -> TokenStream {
    error(
        macro_name,
        &format!("unsupported '{macro_name}' option '{option}'"),
    )
}

fn error(macro_name: &str, message: &str) -> TokenStream {
    tokens::parse(format!(
        "::std::compile_error!({});",
        tokens::string_literal(&format!("{macro_name}: {message}"))
    ))
}
