.PHONY: rebuild_rules
rebuild_rules:
	git submodule update --recursive --init
	cargo run --bin rebuild_rules