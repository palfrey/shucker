Shucker
=======

Shucker is a tracking-param filtering library, designed to strip URLs down to their actual true forms. It contains internally a set of rules derived from the [AdguardFilters](https://github.com/AdguardTeam/AdguardFilters) TrackParamFilter set, and then stripped down be able to be runnable outside of a browser. Note that although the original filters were designed for Javascript-based browser extensions, Shucker's core is a pure-Rust implementation for raw speed (some testing done against Hyperfine, but certainly seems fast enough so far i.e. < 1ms).

There is an [example command line tool provided](src/bin/shuck.rs) (`cargo run --bin shuck <list of urls>`) but the main usage will either be via the `shucker::shuck` fn, or the Python `shucker` library with `shucker.shuck` (which is mostly a thin wrapper over the Rust code), both of which take a URL and return a version of it without the ad-tracking.

Rebuilding the rules set
------------------------
`make rebuild_rules` will pull the latest upstream rules and rebuild.