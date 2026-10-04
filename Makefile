lint:
	cargo fmt --all
	cargo clippy --fix --allow-dirty --all-targets --all-features -- --deny warnings

.PHONY: bump

bump:
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

publish:
	cargo publish --workspace
