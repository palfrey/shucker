use std::env;

use shucker::shuck;

fn main() {
    for argument in env::args().skip(1) {
        let shucked = match shuck(&argument) {
            Ok(val) => val,
            Err(val) => format!("{}", val),
        };
        println!("{argument} => {shucked}");
    }
}
