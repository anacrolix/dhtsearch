serve:
	trunk serve --open

deploy: build
	git checkout -B docs
	git add docs
	git commit -m 'Build docs'
	git push --force-with-lease

build: build-leptos

build-leptos:
	trunk build --dist docs/leptos --release --public-url /leptos/

build-yew:
	trunk build --dist docs/yew --release --public-url /yew/ --features yew --no-default-features