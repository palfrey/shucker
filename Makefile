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
	for i in $(shell seq 9 12); do \
		docker run --rm -v $(PWD)/..:/io ghcr.io/pyo3/maturin build -i 3.$$i --locked --features python; \
	done

develop:
	.venv/bin/maturin develop --locked --uv --features python

testpypi: sync-with-wheel
	.venv/bin/twine upload -r testpypi target/wheels/*.manylinux2014_x86_64.whl target/wheels/*.tar.gz --verbose

pypi: sync-with-wheel
	.venv/bin/twine upload target/wheels/*.manylinux2014_x86_64.whl target/wheels/*.tar.gz

type-check: sync-with-wheel
	.venv/bin/pyright --pythonversion 3.9 --pythonpath $(PWD)/.venv/bin/python tests

watch-type-check: sync-with-wheel
	.venv/bin/pyright --pythonversion 3.9 --pythonpath $(PWD)/.venv/bin/python --watch tests

unittests: sync-with-wheel
	.venv/bin/pytest -vvv tests/

unittests-watch: sync-with-wheel
	.venv/bin/ptw --runner .venv/bin/pytest --now . -- -vvv tests/

requirements.txt: requirements.in constraints.txt
	uv pip compile --no-strip-extras requirements.in -o requirements.txt -c constraints.txt

requirements-with-wheel.txt: requirements.txt requirements-with-wheel.in constraints.txt
	uv pip compile --no-strip-extras requirements-with-wheel.in -o requirements-with-wheel.txt -c constraints.txt

.venv/bin/python:
	uv venv

sync: requirements.txt .venv/bin/python
	uv pip sync requirements.txt

sync-with-wheel: requirements-with-wheel.txt .venv/bin/python
	uv pip sync requirements-with-wheel.txt

pre-commit: sync
	.venv/bin/pre-commit run -a
