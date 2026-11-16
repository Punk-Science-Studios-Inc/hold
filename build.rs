fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let numbers = format!(
        "VERSION_NUM={},{},{},0",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    );
    let text = format!("VERSION_STR=\"{version}\"");
    embed_resource::compile("res/hold.rc", [numbers, text])
        .manifest_required()
        .unwrap();
}
