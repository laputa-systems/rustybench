test:
	python3 tests/bench_driver.py
	cargo test --locked

lint:
	cargo clippy --all-targets --all-features -- --deny warnings

coverage:
	cargo llvm-cov --branch --summary-only --locked

.PHONY: test lint coverage publish

publish:
	./tools/bump_minor_release.sh

DOCKER_IMAGE ?= rustybench:linux-arm64

.PHONY: docker-build-arm64 docker-test-arm64 docker-bench-arm64

docker-build-arm64:
	docker buildx build --platform linux/arm64 --load -t "$(DOCKER_IMAGE)" .

docker-test-arm64: docker-build-arm64
	docker run --rm -v "$(CURDIR):/workspace" -w /workspace "$(DOCKER_IMAGE)" \
		cargo test --workspace --quiet

docker-bench-arm64: docker-build-arm64
	docker run --rm -v "$(CURDIR):/workspace" -w /workspace "$(DOCKER_IMAGE)" \
		cargo bench --bench bench -- --format json --sample-count 1 --sample-size 1

BENCH_ARGS ?=

.PHONY: bench bench-check bench-list
bench:
	python3 tools/bench.py $(BENCH_ARGS)

bench-check:
	python3 tools/bench.py --check $(BENCH_ARGS)

bench-list:
	python3 tools/bench.py --list $(BENCH_ARGS)

.PHONY: clean
clean:
	@if test -L ./target || { test -e ./target && ! test -d ./target; }; then \
	  printf '%s\n' 'make clean: ./target must be a local build directory' >&2; exit 1; \
	fi
	rm -rf -- ./target
