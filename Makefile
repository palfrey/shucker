.PHONY: rebuild_rules
rebuild_rules:
	git submodule -q foreach git pull -q origin master
	cargo run --bin rebuild_rules

.PHONY: build
build: sync
	.venv/bin/maturin build -i 3.12 --locked --features python

build-sdist:
	#docker run --rm -v $(PWD)/..:/io ghcr.io/pyo3/maturin sdist
	.venv/bin/maturin sdist

build-release: sync
	for i in $(shell seq 9 12); do \
		docker run --rm -v $(PWD)/..:/io ghcr.io/pyo3/maturin build -i 3.$$i --locked --features python; \
	done

develop:
	.venv/bin/maturin develop --locked --uv --features python

testpypi: sync
	.venv/bin/twine upload -r testpypi dist/*

pypi: sync
	.venv/bin/twine upload dist/*

type-check: sync
	.venv/bin/pyright --pythonversion 3.9 --pythonpath $(PWD)/.venv/bin/python tests

watch-type-check: sync
	.venv/bin/pyright --pythonversion 3.9 --pythonpath $(PWD)/.venv/bin/python --watch tests

unittests: sync
	.venv/bin/pytest -vvv tests/

unittests-watch: sync
	.venv/bin/ptw --runner .venv/bin/pytest --now . -- -vvv tests/

requirements.txt: requirements.in constraints.txt
	uv pip compile --no-strip-extras requirements.in -o requirements.txt -c constraints.txt

.venv/bin/python:
	uv venv

sync: requirements.txt .venv/bin/python
	uv pip sync requirements.txt

pre-commit: sync
	.venv/bin/pre-commit run -a
