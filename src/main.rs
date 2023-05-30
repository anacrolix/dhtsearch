use url::Url;

mod api;
#[cfg(feature = "leptos")]
mod leptos;
#[cfg(feature = "yew")]
mod yew;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    leptos::mount_to_body()
}

fn make_magnet_link(info_hash: &str) -> String {
    Url::parse_with_params(
        "magnet:",
        &[("xt", format!("urn:btih:{}", info_hash))],
    )
        .unwrap()
        .to_string()
}