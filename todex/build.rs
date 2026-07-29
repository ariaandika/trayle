const LIBS: &[&str] = &["xkbcommon", "libseat"];

fn main() {
    for lib in LIBS {
        pkg_config::Config::new().probe(lib).unwrap();
    }
}
