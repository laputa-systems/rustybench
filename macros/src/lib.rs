use proc_macro::{Delimiter, Ident, TokenStream, TokenTree};

mod attr_options;
mod tokens;

use attr_options::AttrOptions;

#[derive(Clone, Copy)]
pub(crate) enum MacroKind {
    Bench,
    BenchGroup,
}

impl MacroKind {
    fn name(self) -> &'static str {
        match self {
            Self::Bench => "bench",
            Self::BenchGroup => "bench_group",
        }
    }
}

/// The small part of a function signature needed by the registration code.
///
/// The function body and the types of ordinary arguments remain opaque token
/// streams. Rust itself validates those tokens after the generated items are
/// appended.
pub(crate) struct FunctionInfo {
    pub name: Ident,
    pub arg_count: usize,
    pub last_arg_type: Option<TokenStream>,
    pub has_type_param: bool,
    pub has_const_param: bool,
    pub type_before_const: bool,
    pub const_type: Option<TokenStream>,
    pub is_extern_abi: bool,
    pub ignored: bool,
}

#[derive(Default)]
struct GenericInfo {
    type_index: Option<usize>,
    const_index: Option<usize>,
    const_type: Option<TokenStream>,
}

#[proc_macro_attribute]
pub fn bench(options: TokenStream, item: TokenStream) -> TokenStream {
    let function = match parse_function(&item) {
        Ok(function) => function,
        Err(error) => return error,
    };

    let options = match AttrOptions::parse(options, MacroKind::Bench, Some(&function)) {
        Ok(options) => options,
        Err(error) => return error,
    };

    let generated = generate_bench(&function, &options);
    let mut result = item;
    result.extend(generated);
    result
}

#[proc_macro_attribute]
pub fn bench_group(options: TokenStream, item: TokenStream) -> TokenStream {
    let module = match parse_module(&item) {
        Ok(module) => module,
        Err(error) => return error,
    };

    let options = match AttrOptions::parse(options, MacroKind::BenchGroup, None) {
        Ok(options) => options,
        Err(error) => return error,
    };

    let private_mod = &options.private_mod;
    let static_ident = format!(
        "__RUSTYBENCH_GROUP_{}",
        pretty_name(&module.name.to_string()).to_uppercase()
    );
    let meta = entry_meta_expr(&module.name.to_string(), &options, module.ignored);
    let generated = format_tokens(format!(
        "{} static {static_ident}: {private_mod}::EntryList<{private_mod}::GroupEntry> = {{ {{ {} static PUSH: extern \"C\" fn() = push; extern \"C\" fn push() {{ {private_mod}::GROUP_ENTRIES.push(&{static_ident}); }} }} {private_mod}::EntryList::new({{ static {static_ident}: {private_mod}::GroupEntry = {private_mod}::GroupEntry {{ meta: {meta}, generic_benches: None, }}; &{static_ident} }}) }};",
        unsupported_error("bench_group"),
        pre_main_attrs()
    ));

    let mut result = item;
    result.extend(generated);
    result
}

fn generate_bench(function: &FunctionInfo, options: &AttrOptions) -> TokenStream {
    let private_mod = &options.private_mod;
    let static_ident = format!(
        "__RUSTYBENCH_BENCH_{}",
        pretty_name(&function.name.to_string()).to_uppercase()
    );
    let meta = entry_meta_expr(&function.name.to_string(), options, function.ignored);
    let bench_args_global = if options.args_expr.is_some() {
        format!(
            "static __RUSTYBENCH_ARGS: {private_mod}::BenchArgs = {private_mod}::BenchArgs::new();"
        )
    } else {
        String::new()
    };

    let last_arg_type = options
        .args_expr
        .as_ref()
        .and(function.last_arg_type.as_ref())
        .map(tokens::display)
        .unwrap_or_default();
    let arg_return = options
        .args_expr
        .as_ref()
        .filter(|args| args.is_empty_array())
        .map(|_| format!("-> [{last_arg_type}; 0]"))
        .unwrap_or_default();

    let make_bench_fn = |generics: &[String]| {
        let mut function_expr = if generics.is_empty() {
            function.name.to_string()
        } else {
            format!("{}::<{}>", function.name, generics.join(", "))
        };

        match (function.arg_count, options.args_expr.as_ref()) {
            (0, None) => {
                if function.is_extern_abi {
                    function_expr = format!("|| {function_expr}()");
                }
                format!(
                    "{private_mod}::BenchEntryRunner::Plain(|rustybench| rustybench.bench({function_expr}))"
                )
            }
            (0, Some(_)) => unreachable!(),
            (1, None) => {
                if function.is_extern_abi {
                    function_expr = format!("|rustybench| {function_expr}(rustybench)");
                }
                format!("{private_mod}::BenchEntryRunner::Plain({function_expr})")
            }
            (1, Some(args)) => format!(
                "{private_mod}::BenchEntryRunner::Args(|| __RUSTYBENCH_ARGS.runner(|| {arg_return} {{ {} }}, |arg| {private_mod}::ToStringHelper(arg).to_string(), |rustybench, __rustybench_arg| rustybench.bench(|| {function_expr}({private_mod}::Arg::<{last_arg_type}>::get(__rustybench_arg)))))",
                tokens::display(&args.raw)
            ),
            (2, Some(args)) => format!(
                "{private_mod}::BenchEntryRunner::Args(|| __RUSTYBENCH_ARGS.runner(|| {arg_return} {{ {} }}, |arg| {private_mod}::ToStringHelper(arg).to_string(), |rustybench, __rustybench_arg| {function_expr}(rustybench, {private_mod}::Arg::<{last_arg_type}>::get(__rustybench_arg))))",
                tokens::display(&args.raw)
            ),
            (_, None) => format!(
                "::std::compile_error!({})",
                tokens::string_literal(&format!(
                    "expected 'args' option containing '{last_arg_type}'"
                ))
            ),
            (_, Some(_)) => unreachable!(),
        }
    };

    let make_generic_entry = |ty: Option<&TokenStream>, const_value: Option<String>| {
        let generic_const = const_value.as_ref().map(|value| format!("{{{value}}}"));
        let mut generics = Vec::new();
        if let Some(value) = generic_const {
            generics.push(value);
        }
        if let Some(ty) = ty {
            generics.push(tokens::display(ty));
        }
        if function.type_before_const {
            generics.reverse();
        }

        let bench_fn = make_bench_fn(&generics);
        let type_value = ty.map(|ty| {
            format!(
                "Some({private_mod}::EntryType::new::<{}>())",
                tokens::display(ty)
            )
        });
        let const_value = const_value
            .as_ref()
            .map(|value| format!("Some({private_mod}::EntryConst::new(&{value}))"));

        format!(
            "{private_mod}::GenericBenchEntry {{ group: &{static_ident}, bench: {bench_fn}, ty: {}, const_value: {} }}",
            type_value.unwrap_or_else(|| "None".to_owned()),
            const_value.unwrap_or_else(|| "None".to_owned())
        )
    };

    // Validate constant values even when another generic dimension is empty. In
    // that case there are no generated benchmark entries to carry the type
    // check, but an invalid `consts` value must still be rejected.
    let const_validation = options
        .generic
        .consts
        .as_ref()
        .map(|consts| {
            let const_type = function.const_type.as_ref().unwrap();
            format!(
                "const _: &[{}] = &{};",
                tokens::display(const_type),
                tokens::display(&consts.raw)
            )
        })
        .unwrap_or_default();

    let generated_items = if options.generic.is_empty() {
        String::new()
    } else {
        match options.generic.consts.as_ref() {
            None => match options.generic.types.as_ref() {
                None => {
                    let bench_fn = make_bench_fn(&[]);
                    let entry =
                        format!("{private_mod}::BenchEntry {{ meta: {meta}, bench: {bench_fn} }}");
                    format!(
                        "{} static {static_ident}: {private_mod}::BenchEntry = {{ {{ {} static PUSH: extern \"C\" fn() = push; extern \"C\" fn push() {{ static NODE: {private_mod}::EntryList<{private_mod}::BenchEntry> = {private_mod}::EntryList::new(&{static_ident}); {private_mod}::BENCH_ENTRIES.push(&NODE); }} }} {bench_args_global} {entry} }};",
                        unsupported_error("bench"),
                        pre_main_attrs()
                    )
                }
                Some(types) => {
                    let entries = types
                        .iter()
                        .map(|ty| make_generic_entry(Some(ty), None))
                        .collect::<Vec<_>>()
                        .join(",");
                    make_generic_group(
                        private_mod,
                        &static_ident,
                        &meta,
                        &bench_args_global,
                        format!("&[&[{entries}]]"),
                    )
                }
            },
            Some(consts) if consts.array.is_some() => {
                let values = consts.array.as_ref().unwrap();
                let const_type = function.const_type.as_ref().unwrap();
                let groups = options
                    .generic
                    .types_iter()
                    .map(|ty| {
                        let entries = (0..values.len())
                            .map(|index| {
                                make_generic_entry(
                                    ty,
                                    Some(format!("__RUSTYBENCH_CONSTS[{index}]")),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(
                            "{{ static __RUSTYBENCH_GENERIC_BENCHES: [{private_mod}::GenericBenchEntry; {}] = [{entries}]; &__RUSTYBENCH_GENERIC_BENCHES }}",
                            values.len()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                make_generic_group(
                    private_mod,
                    &static_ident,
                    &meta,
                    &bench_args_global,
                    format!(
                        "{{ const __RUSTYBENCH_CONSTS: &[{}] = &{}; &[{groups}] }}",
                        tokens::display(const_type),
                        tokens::display(&consts.raw)
                    ),
                )
            }
            Some(consts) => {
                const MAX_EXTERN_COUNT: usize = 20;
                let const_type = function.const_type.as_ref().unwrap();
                let groups = options
                    .generic
                    .types_iter()
                    .map(|ty| {
                        let entries = (0..MAX_EXTERN_COUNT)
                            .map(|index| {
                                make_generic_entry(
                                    ty,
                                    Some(format!(
                                        "__RUSTYBENCH_CONSTS[if {index} < __RUSTYBENCH_CONST_COUNT {{ {index} }} else {{ 0 }}]"
                                    )),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(
                            "{{ static __RUSTYBENCH_GENERIC_BENCHES: [{private_mod}::GenericBenchEntry; __RUSTYBENCH_CONST_COUNT] = match {private_mod}::shrink_array([{entries}]) {{ Some(array) => array, _ => panic!(\"external 'consts' cannot contain more than 20 values\"), }}; &__RUSTYBENCH_GENERIC_BENCHES }}"
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                make_generic_group(
                    private_mod,
                    &static_ident,
                    &meta,
                    &bench_args_global,
                    format!(
                        "{{ const __RUSTYBENCH_CONST_COUNT: usize = __RUSTYBENCH_CONSTS.len(); const __RUSTYBENCH_CONSTS: &[{}] = &{}; &[{groups}] }}",
                        tokens::display(const_type),
                        tokens::display(&consts.raw)
                    ),
                )
            }
        }
    };

    format_tokens(format!("{const_validation}{generated_items}"))
}

fn make_generic_group(
    private_mod: &str,
    static_ident: &str,
    meta: &str,
    bench_args_global: &str,
    generic_benches: String,
) -> String {
    format!(
        "{} static {static_ident}: {private_mod}::GroupEntry = {{ {{ {} static PUSH: extern \"C\" fn() = push; extern \"C\" fn push() {{ static NODE: {private_mod}::EntryList<{private_mod}::GroupEntry> = {private_mod}::EntryList::new(&{static_ident}); {private_mod}::GROUP_ENTRIES.push(&NODE); }} }} {bench_args_global} {private_mod}::GroupEntry {{ meta: {meta}, generic_benches: Some({generic_benches}) }} }};",
        unsupported_error("bench"),
        pre_main_attrs()
    )
}

struct ModuleInfo {
    name: Ident,
    ignored: bool,
}

fn parse_function(item: &TokenStream) -> Result<FunctionInfo, TokenStream> {
    let tokens: Vec<_> = item.clone().into_iter().collect();
    let ignored = has_ignore_attribute(&tokens);
    let Some(fn_index) = tokens
        .iter()
        .position(|token| tokens::is_ident(token, "fn"))
    else {
        return Err(parse_error("expected a function item"));
    };
    let Some(TokenTree::Ident(name)) = tokens.get(fn_index + 1) else {
        return Err(parse_error("expected a function name"));
    };
    let Some((args_index, args_group)) =
        tokens[fn_index + 2..]
            .iter()
            .enumerate()
            .find_map(|(index, token)| {
                tokens::group(token, Delimiter::Parenthesis)
                    .map(|group| (index + fn_index + 2, group))
            })
    else {
        return Err(parse_error("expected function arguments"));
    };

    let generic = parse_generics(&tokens[fn_index + 2..args_index]);
    let args: Vec<_> =
        tokens::split_top_level(&args_group.stream().into_iter().collect::<Vec<_>>(), ',')
            .into_iter()
            .filter(|arg| !arg.is_empty())
            .collect();
    let last_arg_type = args.last().and_then(|arg| argument_type(arg));
    let is_extern_abi = tokens[..fn_index]
        .iter()
        .any(|token| tokens::is_ident(token, "extern"));

    Ok(FunctionInfo {
        name: name.clone(),
        arg_count: args.len(),
        last_arg_type,
        has_type_param: generic.type_index.is_some(),
        has_const_param: generic.const_index.is_some(),
        type_before_const: matches!((generic.type_index, generic.const_index), (Some(ty), Some(c)) if ty < c),
        const_type: generic.const_type,
        is_extern_abi,
        ignored,
    })
}

fn parse_module(item: &TokenStream) -> Result<ModuleInfo, TokenStream> {
    let tokens: Vec<_> = item.clone().into_iter().collect();
    let ignored = has_ignore_attribute(&tokens);
    let Some(mod_index) = tokens
        .iter()
        .position(|token| tokens::is_ident(token, "mod"))
    else {
        return Err(parse_error("expected a module item"));
    };
    let Some(TokenTree::Ident(name)) = tokens.get(mod_index + 1) else {
        return Err(parse_error("expected a module name"));
    };
    Ok(ModuleInfo {
        name: name.clone(),
        ignored,
    })
}

fn parse_generics(tokens: &[TokenTree]) -> GenericInfo {
    let mut generic = GenericInfo::default();
    let Some(start) = tokens.iter().position(|token| tokens::is_punct(token, '<')) else {
        return generic;
    };
    let mut depth = 0;
    let mut content = Vec::new();
    for token in &tokens[start..] {
        if tokens::is_punct(token, '<') {
            depth += 1;
            if depth > 1 {
                content.push(token.clone());
            }
        } else if tokens::is_punct(token, '>') {
            depth -= 1;
            if depth == 0 {
                break;
            }
            content.push(token.clone());
        } else if depth > 0 {
            content.push(token.clone());
        }
    }

    for (index, parameter) in tokens::split_top_level(&content, ',')
        .into_iter()
        .enumerate()
    {
        if parameter.is_empty() {
            continue;
        }
        if tokens::is_ident(&parameter[0], "const") {
            if generic.const_index.is_none() {
                generic.const_index = Some(index);
                if let Some(colon) = parameter
                    .iter()
                    .position(|token| tokens::is_punct(token, ':'))
                {
                    generic.const_type =
                        Some(tokens::token_stream(parameter[colon + 1..].iter().cloned()));
                }
            }
        } else if matches!(parameter[0], TokenTree::Ident(_)) && generic.type_index.is_none() {
            generic.type_index = Some(index);
        }
    }
    generic
}

fn argument_type(argument: &[TokenTree]) -> Option<TokenStream> {
    if let Some(colon) = argument
        .iter()
        .position(|token| tokens::is_punct(token, ':'))
    {
        return Some(strip_reference_lifetime(tokens::token_stream(
            argument[colon + 1..].iter().cloned(),
        )));
    }

    if tokens::is_ident(argument.first()?, "self") {
        return Some(tokens::parse("Self"));
    }
    if tokens::is_punct(argument.first()?, '&') {
        let is_mut = argument.iter().any(|token| tokens::is_ident(token, "mut"));
        return Some(tokens::parse(if is_mut { "&mut Self" } else { "&Self" }));
    }
    None
}

fn strip_reference_lifetime(value: TokenStream) -> TokenStream {
    let tokens: Vec<_> = value.into_iter().collect();
    if tokens
        .first()
        .is_some_and(|token| tokens::is_punct(token, '&'))
        && tokens
            .get(1)
            .is_some_and(|token| tokens::is_punct(token, '\''))
    {
        return tokens::token_stream(
            tokens
                .into_iter()
                .enumerate()
                .filter(|(index, _)| *index != 1 && *index != 2)
                .map(|(_, token)| token),
        );
    }
    tokens::token_stream(tokens)
}

fn has_ignore_attribute(tokens: &[TokenTree]) -> bool {
    let mut index = 0;
    while index + 1 < tokens.len() {
        if tokens::is_punct(&tokens[index], '#')
            && let Some(group) = tokens::group(&tokens[index + 1], Delimiter::Bracket)
        {
            if group
                .stream()
                .into_iter()
                .next()
                .is_some_and(|token| tokens::is_ident(&token, "ignore"))
            {
                return true;
            }
            index += 2;
            continue;
        }
        if tokens::is_ident(&tokens[index], "fn") || tokens::is_ident(&tokens[index], "mod") {
            break;
        }
        index += 1;
    }
    false
}

/// Lists of comma-separated `#[cfg]` parameters.
fn pre_main_attrs() -> String {
    "#[used] #[cfg_attr(windows, unsafe(link_section = \".CRT$XCU\"))] #[cfg_attr(any(target_os = \"android\", target_os = \"dragonfly\", target_os = \"freebsd\", target_os = \"fuchsia\", target_os = \"haiku\", target_os = \"illumos\", target_os = \"linux\", target_os = \"netbsd\", target_os = \"openbsd\", target_os = \"wasi\", target_os = \"emscripten\"), unsafe(link_section = \".init_array\"))] #[cfg_attr(any(target_os = \"ios\", target_os = \"macos\", target_os = \"tvos\", target_os = \"watchos\"), unsafe(link_section = \"__DATA,__mod_init_func,mod_init_funcs\"))]".to_owned()
}

fn unsupported_error(attr_name: &str) -> String {
    format!(
        "#[cfg(not(any(windows, target_os = \"android\", target_os = \"dragonfly\", target_os = \"freebsd\", target_os = \"fuchsia\", target_os = \"haiku\", target_os = \"illumos\", target_os = \"linux\", target_os = \"netbsd\", target_os = \"openbsd\", target_os = \"wasi\", target_os = \"emscripten\", target_os = \"ios\", target_os = \"macos\", target_os = \"tvos\", target_os = \"watchos\"))) ] ::std::compile_error!({});",
        tokens::string_literal(&format!(
            "Unsupported target OS for `#[rustybench::{attr_name}]`"
        ))
    )
}

fn entry_meta_expr(raw_name: &str, options: &AttrOptions, ignored: bool) -> String {
    let display_name = options
        .name_expr
        .as_ref()
        .map(tokens::display)
        .unwrap_or_else(|| tokens::string_literal(pretty_name(raw_name)));
    format!(
        "{}::EntryMeta {{ raw_name: {}, display_name: {}, bench_options: {}, module_path: ::std::module_path!(), location: {}::EntryLocation {{ file: ::std::file!(), line: ::std::line!(), col: ::std::column!(), }} }}",
        options.private_mod,
        tokens::string_literal(raw_name),
        display_name,
        tokens::display(&options.bench_options_fn(ignored)),
        options.private_mod
    )
}

fn pretty_name(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

fn format_tokens(source: String) -> TokenStream {
    tokens::parse(source)
}

fn parse_error(message: &str) -> TokenStream {
    format_tokens(format!(
        "::std::compile_error!({});",
        tokens::string_literal(message)
    ))
}
