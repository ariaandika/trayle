const LIBS: &[&str] = &["xkbcommon", "libseat", "libudev"];

fn main() {
    for lib in LIBS {
        pkg_config::Config::new().probe(lib).unwrap();
    }
}
