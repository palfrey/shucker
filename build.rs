use std::{
    collections::BTreeMap,
    fmt::Debug,
    fs::{self, read_to_string, File},
    io::Write,
    path::Path,
};

use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

fn skip_regex_splitter(params: &str) -> Vec<&str> {
    let mut ret = vec![];
    let mut in_regex = false;
    let mut first_index = 0;
    for (index, c) in params.chars().enumerate() {
        if !in_regex {
            match c {
                '/' => {
                    in_regex = true;
                }
                ',' => {
                    ret.push(&params[first_index..index]);
                    first_index = index + 1;
                }
                _ => {}
            }
        } else {
            match c {
                '/' => {
                    in_regex = false;
                }
                _ => {}
            }
        }
    }
    if first_index < params.len() {
        ret.push(&params[first_index..])
    }
    return ret;
}

#[derive(Debug, Clone)]
enum Command {
    Hostname(String),
    RemoveParamAll,
    RemoveParam(String),
    RemoveParamRegex(String),
    Domain(Vec<String>),
    Invert,
}

fn build_commands(out_dir: &Path) -> Result<Vec<Vec<Command>>> {
    let text_dump_path = out_dir.join("test.txt");
    let raw_rules: String = read_to_string("src/rules.txt").unwrap();
    let mut text_dump = File::create(text_dump_path).unwrap();
    let mut all_commands: Vec<Vec<Command>> = vec![];
    for (mut line_index, mut line) in raw_rules.split('\n').map(|l| l.trim_end()).enumerate() {
        line_index += 1;
        if line.len() == 0
            || line.starts_with('!')
            || line.contains("##.")
            || line.contains("#$#")
            || line.contains("https-filtering-check.adtidy.org")
        {
            continue;
        }
        let mut commands: Vec<Command> = vec![];
        if line.starts_with("@@") {
            commands.push(Command::Invert);
            line = &line[2..];
        }
        if !line.starts_with("$") {
            let (hostname, _rest) = line.split_once('$').expect(&format!("hostname: '{line}'"));
            commands.push(Command::Hostname(hostname.into()));
            line = &line[(hostname.len())..];
        }
        assert!(line.starts_with("$"), "Missing $ at {line_index}");
        line = &line[1..];

        for p in skip_regex_splitter(line) {
            match p {
                "~third-party" | "third-party" | "xmlhttprequest" | "image" | "document"
                | "media" | "script" | "subdocument" | "font" | "stylesheet" => {
                    continue;
                }
                "removeparam" => {
                    // no args version
                    commands.push(Command::RemoveParamAll);
                    continue;
                }
                _ => {}
            }
            let (key, value) = p
                .split_once('=')
                .expect(&format!("p: '{p}' at {line_index}"));
            match key {
                "cookie" | "app" | "denyallow" => {}
                "domain" => {
                    commands.push(Command::Domain(
                        value.split('|').map(String::from).collect(),
                    ));
                }
                "removeparam" => {
                    if value.starts_with("/") {
                        commands.push(Command::RemoveParamRegex(String::from(value)));
                    } else {
                        commands.push(Command::RemoveParam(String::from(value)));
                    }
                }
                key => {
                    panic!("key: {key} value: {value} at {line_index}");
                }
            }
        }
        text_dump.write(format!("command: {commands:#?}").as_bytes())?;
        text_dump.write("\n".as_bytes())?;
        all_commands.push(commands);
    }
    Ok(all_commands)
}

fn comment_block(value: impl Debug) -> Vec<TokenStream> {
    let raw_comment = format!("{value:#?}");

    return raw_comment
        .lines()
        .map(|line| quote!(#[doc = #line]))
        .collect();
}

fn generate_host_filters(value: &Vec<Command>) -> Vec<TokenStream> {
    let mut requirements = vec![];
    for command in value {
        match command {
            Command::Hostname(hostname) => {
                let mut hostname_pattern = hostname.clone();
                if hostname_pattern.starts_with("||") {
                    hostname_pattern = hostname_pattern.replace("||", "https?://(?:www\\.)?")
                }
                if hostname_pattern.ends_with("^") {
                    hostname_pattern = hostname_pattern.replace("^", "[^a-z0-9_\\-\\.%]")
                }
                requirements.push(quote! {
                    Regex::new(#hostname_pattern).unwrap().is_match(url_str)
                });
            }
            Command::Domain(domains) => {
                requirements.push(quote! {
                    url.host_str().map(|h| {
                        #(
                           if h == #domains { return true;}
                           if h.ends_with(concat!(".", #domains)) { return true;}
                        )*
                        return false;
                        }).unwrap_or(false)
                });
            }
            _ => {}
        }
    }
    return requirements;
}

fn build_remove_params(all_commands: &Vec<Vec<Command>>) -> TokenStream {
    let mut remove_params: BTreeMap<String, Vec<Vec<Command>>> = BTreeMap::new();
    all_commands
        .iter()
        .map(|cv| {
            let remove_param_keys = cv
                .iter()
                .map(|c| match c {
                    Command::RemoveParam(param) => Some(param),
                    _ => None,
                })
                .filter(|p| p.is_some())
                .next();
            if let Some(key) = remove_param_keys {
                (key, cv)
            } else {
                (None, cv)
            }
        })
        .filter(|(key, _)| key.is_some())
        .map(|(key, value)| (key.unwrap(), value))
        .for_each(|(key, value)| {
            remove_params
                .entry(key.clone())
                .and_modify(|v| v.push(value.to_vec()))
                .or_insert(vec![value.to_vec()]);
        });
    let mut patterns = vec![];
    for (key, value) in remove_params.iter() {
        let mut requirements = vec![];
        let mut has_no_filter_command = false;
        for commands in value {
            if commands.len() == 1 {
                if let Command::RemoveParam(_) = commands.get(0).unwrap() {
                    // Only command is remove param, so just remove it
                    has_no_filter_command = true;
                }
            }

            requirements.extend(generate_host_filters(commands));
        }
        if has_no_filter_command || requirements.is_empty() {
            requirements.push(quote! {true});
        }
        let comments = comment_block(value);

        let matcher = quote! {
            #key => {
                #(#comments)*
                #(if #requirements {continue;})*
            }
        };
        patterns.push(matcher);
    }
    return quote! {
        match key.deref() {
            #( #patterns )*
            _ => {}
        }
    };
}

fn build_remove_params_regex(all_commands: &Vec<Vec<Command>>) -> TokenStream {
    let mut checks = vec![];
    all_commands
        .iter()
        .map(|cv| {
            let remove_param_keys = cv
                .iter()
                .map(|c| match c {
                    Command::RemoveParamRegex(param) => Some(param),
                    _ => None,
                })
                .filter(|p| p.is_some())
                .next();
            if let Some(key) = remove_param_keys {
                (key, cv)
            } else {
                (None, cv)
            }
        })
        .filter(|(key, _)| key.is_some())
        .map(|(key, value)| (key.unwrap(), value))
        .for_each(|(key, value)| {
            let check_key = key.replace("\\", "");
            let without_slashes = &check_key[1..check_key.len() - 1];
            let comments: Vec<TokenStream> = comment_block(value);
            let requirements = generate_host_filters(&value);
            checks.push(quote!( {
                #(#comments)*
                if Regex::new(#without_slashes).unwrap().is_match(&key) {
                    #(if #requirements {continue;})*
                }
            }));
        });
    return quote!({
        #(#checks)*
    });
}

fn main() -> Result<()> {
    let out_dir_var = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_var);
    let rust_stripper_path = out_dir.join("rules_generated.rs");
    let all_commands = build_commands(out_dir)?;
    let remove_param_check = build_remove_params(&all_commands);
    let remove_param_regex_check = build_remove_params_regex(&all_commands);
    let output = quote! {
       use url::Url;
       use anyhow::Result;
       use std::ops::Deref;
       use regex::Regex;
       pub fn stripper(url_str: &str) -> Result<String> {
        let mut url = Url::parse(url_str)?;
        let mut query: Vec<(String, String)> = vec![];
        for (key, value) in url.query_pairs() {
            #remove_param_check
            #remove_param_regex_check
            query.push((key.to_string(), value.to_string()));
        }
        if query.is_empty() {
            url.set_query(None)
        } else {
            url.query_pairs_mut().clear().extend_pairs(query);
        }
        Ok(url.into())
       }

    };
    fs::write(Path::new(&out_dir).join("debug.rs"), format!("{output:#?}")).unwrap();
    let syntax_tree = syn::parse2(output).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    fs::write(rust_stripper_path, &formatted)?;
    Ok(())
}
