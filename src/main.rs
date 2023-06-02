use std::sync::Arc;

mod api;
#[cfg(feature = "leptos")]
mod leptos;
#[cfg(feature = "yew")]
mod yew;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    #[cfg(feature = "leptos")]
    leptos::mount_to_body();
    #[cfg(feature = "yew")]
    yew::mount_to_body();
}

fn make_magnet_link(info_hash: &str) -> String {
    "magnet:?xt=urn:btih:".to_owned() + info_hash
}

pub type Result<T> = std::result::Result<T, Arc<Error>>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GlooNet(#[from] gloo_net::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
