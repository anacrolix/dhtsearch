serve:
	trunk serve --open

build: build-leptos build-yew

build-leptos:
	trunk build --dist docs/leptos --release --public-url /leptos/

build-yew:
	trunk build --dist docs/yew --release --public-url /yew/ --features yew --no-default-features