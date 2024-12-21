use std::{
    fs::{read_to_string, File},
    io::Write,
};

use anyhow::Error;

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
    Domain(Vec<String>),
    DenyAllow(Vec<String>),
    App(String),
    Invert,
}

fn main() -> Result<(), Error> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("test.txt");
    let raw_rules = read_to_string("src/rules.txt").unwrap();
    let mut f = File::create(destination).unwrap();
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
                    commands.push(Command::RemoveParam(String::from(value)));
                }
                "app" => {
                    commands.push(Command::App(String::from(value)));
                }
                key => {
                    panic!("key: {key} value: {value} at {line_index}");
                }
            }
        }
        f.write(format!("command: {commands:#?}").as_bytes())?;
        f.write("\n".as_bytes())?;
    }
    Ok(())
}
