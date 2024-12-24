use std::{
    fs::{self, read_to_string, File},
    io::Write,
};

use anyhow::Error;
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

#[derive(Debug)]
enum Command {
    Hostname(String),
    ThirdParty(bool),
    RemoveParamAll,
    RemoveParam(String),
    RemoveParamRegex(String),
    Domain(Vec<String>),
    DenyAllow(Vec<String>),
    App(String),
    Invert,
}

fn main() -> Result<(), Error> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let text_dump_path = std::path::Path::new(&out_dir).join("test.txt");
    let rust_stripper_path = std::path::Path::new(&out_dir).join("rules_generated.rs");
    let raw_rules: String = read_to_string("src/rules.txt").unwrap();
    let mut text_dump = File::create(text_dump_path).unwrap();
    let mut all_commands: Vec<Vec<Command>> = vec![];
    for (mut line_index, mut line) in raw_rules.split('\n').map(|l| l.trim_end()).enumerate() {
        line_index += 1;
        if line.len() == 0 || line.starts_with('!') {
            continue;
        }
        let mut commands: Vec<Command> = vec![];
        if line.starts_with("@@") {
            commands.push(Command::Invert);
            line = &line[2..];
        }
        if !line.starts_with("$") {
            let (hostname, _rest) = line[2..]
                .split_once('$')
                .expect(&format!("hostname: '{line}'"));
            commands.push(Command::Hostname(hostname.into()));
            line = &line[(hostname.len() + 2)..];
        }
        assert!(line.starts_with("$"), "Missing $ at {line_index}");
        line = &line[1..];

        for p in skip_regex_splitter(line) {
            match p {
                "~third-party" => {
                    commands.push(Command::ThirdParty(false));
                    continue;
                }
                "third-party" => {
                    commands.push(Command::ThirdParty(true));
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
                "domain" => {
                    commands.push(Command::Domain(
                        value.split('|').map(String::from).collect(),
                    ));
                }
                "denyallow" => {
                    commands.push(Command::DenyAllow(
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
                "app" => {
                    commands.push(Command::App(String::from(value)));
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
    let always_delete: Vec<String> = all_commands
        .iter()
        .filter(|c| {
            if c.len() != 1 {
                return false;
            }
            match c.first().unwrap() {
                Command::RemoveParam(_) => true,
                _ => false,
            }
        })
        .map(|c| match c.first().unwrap() {
            Command::RemoveParam(param) => param,
            _ => panic!("Should be impossible due to filter"),
        })
        .cloned()
        .collect();
    let output = quote! {
       use url::Url;
       use anyhow::Result;
       use std::collections::HashMap;
       use std::ops::Deref;
       pub fn stripper(url_str: &str) -> Result<String> {
        let mut url = Url::parse(url_str)?;
        let mut query: HashMap<String, String> = HashMap::new();
        for (key, value) in url.query_pairs() {
            match key.deref() {
                #( #always_delete  => {} )*
                _ => {
                    query.insert(key.to_string(), value.to_string());
                }
            }
        }
        if query.is_empty() {
            url.set_query(None)
        } else {
            url.query_pairs_mut().clear().extend_pairs(query);
        }
        Ok(url.into())
       }

    };
    let syntax_tree = syn::parse2(output).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    fs::write(rust_stripper_path, formatted)?;
    Ok(())
}
