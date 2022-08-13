const OUT_DIR: &str = "fourth-target";
fn main() {
    if let Err(e) = fs::create_dir(OUT_DIR) {
        eprintln!("{:?}", e);
    };
    std::env::set_var("OUT_DIR", OUT_DIR);
}
