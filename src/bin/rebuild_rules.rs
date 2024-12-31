use itertools::join;
use std::{fs, path::Path};

fn main() {
    let sources = [
        "external/adguardfilters/TrackParamFilter/sections/allowlist.txt",
        "external/adguardfilters/TrackParamFilter/sections/general_url.txt",
        "external/adguardfilters/TrackParamFilter/sections/specific.txt",
    ];
    println!("Rebuilding...");
    let all_data = format!(
        concat!(
            "! Content reused from https://github.com/AdguardTeam/AdguardFilters under GPLv3\n",
            "! Run `cargo run --bin rebuild_rules` to rebuild\n",
            "!\n",
            "{}"
        ),
        join(
            sources
                .iter()
                .map(|p| format!("! Content from {p}\n{}", fs::read_to_string(p).unwrap())),
            "\n",
        )
    );
    let rules_path = Path::new("src/rules.txt");
    let existing_rules = fs::read_to_string(rules_path).unwrap();
    if existing_rules != all_data {
        println!("Writing changes");
        fs::write(rules_path, all_data).unwrap();
    }
    println!("Rebuilt");
}
