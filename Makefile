.PHONY: rebuild_rules
rebuild_rules:
	git submodule -q foreach git pull -q origin master
	cargo run --bin rebuild_rules