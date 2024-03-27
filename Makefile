.PHONY: help build-test-contracts

help:             ## Show the help.
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@fgrep "##" Makefile | fgrep -v fgrep

build-test-contracts:   ## Build the test contracts and fix the generated metadata
	bash ./scripts/build-test-contracts.sh