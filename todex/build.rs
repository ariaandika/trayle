const LIBS: &[&str] = &["xkbcommon"];

fn main() {
    for lib in LIBS {
        pkg_config::Config::new().probe(lib).unwrap();
    }
}
