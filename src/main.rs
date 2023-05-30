mod api;
mod leptos;
mod yew;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    leptos::mount_to_body()
}
