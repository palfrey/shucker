.PHONY: build
build: sync
	.venv/bin/maturin build -i 3.12 -m ../Cargo.toml --locked

testpypi: sync
	.venv/bin/twine upload -r testpypi dist/*

pypi: sync
	.venv/bin/twine upload dist/*

type-check: sync
	.venv/bin/pyright --pythonversion 3.9 --pythonpath $(PWD)/.venv/bin/python paracrine integration_test tests

watch-type-check: sync
	.venv/bin/pyright --pythonversion 3.9 --pythonpath $(PWD)/.venv/bin/python --watch paracrine integration_test tests

unittests-watch: sync
	.venv/bin/ptw --runner .venv/bin/pytest --now . -- -vvv tests/

requirements.txt: requirements.in constraints.txt
	uv pip compile --no-strip-extras requirements.in -o requirements.txt -c constraints.txt

.venv/bin/python:
	uv venv

sync: requirements.txt .venv/bin/python
	uv pip sync requirements.txt

pre-commit: sync
	./.venv/bin/pre-commit run -a

develop:
	./.venv/bin/maturin develop -m ../Cargo.toml --locked --uv