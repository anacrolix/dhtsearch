serve-args := "--port 8081"

serve:
    trunk serve --open {{ serve-args }}

serve-release:
    trunk serve --open --release {{ serve-args }}

deploy: build
    git checkout -B docs
    git add docs
    git commit -m 'Build docs'
    git push --force-with-lease

build: build-leptos

build-leptos:
    #!/bin/bash
    trunk build --dist docs --release
    echo -n wasm.dht.lol > docs/CNAME

build-yew:
    trunk build --dist docs/yew --release --public-url /yew/ --features yew --no-default-features
