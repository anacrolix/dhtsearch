fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dhtsearch::mount_to_body();
}
