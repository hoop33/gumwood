.PHONY: default
default: coverage

.PHONY: coverage
coverage:
	cargo tarpaulin -v

html_coverage:
	cargo tarpaulin -o Html
	xdg-open tarpaulin-report.html

.PHONY: deps
deps:
	cargo install cargo-tarpaulin
