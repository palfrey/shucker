.PHONY: rebuild_rules
rebuild_rules:
	git submodule -q foreach git pull -q origin master
	cargo run --bin rebuild_rules

.PHONY: build
build: sync
	.venv/bin/maturin build -i 3.12 --locked --features python

build-sdist: sync
	.venv/bin/maturin sdist

build-release: sync
	for i in $(shell seq 11 13); do \
		docker run --rm -v $(PWD):/io ghcr.io/pyo3/maturin build -i 3.$$i --locked --features python --release; \
	done

develop:
	.venv/bin/maturin develop --locked --uv --features python

testpypi: sync-with-wheel
	.venv/bin/twine upload -r testpypi target/wheels/*.manylinux2014_x86_64.whl target/wheels/*.tar.gz --verbose

pypi: sync
	.venv/bin/twine upload target/wheels/*.manylinux2014_x86_64.whl target/wheels/*.tar.gz

type-check: sync-with-wheel
	.venv/bin/pyright --pythonversion 3.11 --pythonpath $(PWD)/.venv/bin/python tests

watch-type-check: sync-with-wheel
	.venv/bin/pyright --pythonversion 3.11 --pythonpath $(PWD)/.venv/bin/python --watch tests

unittests: sync-with-wheel
	.venv/bin/pytest -vvv tests/

unittests-watch: sync-with-wheel
	.venv/bin/ptw --runner .venv/bin/pytest --now . -- -vvv tests/

.venv/bin/python:
	uv venv

sync: .venv/bin/python
	uv sync

sync-with-wheel: sync
	uv pip install target/wheels/shucker-0.1.1-cp312-cp312-manylinux_2_34_x86_64.whl

pre-commit: sync
	.venv/bin/pre-commit run -a
