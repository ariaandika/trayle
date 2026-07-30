const LIBS: &[&str] = &["xkbcommon", "libseat", "libudev", "libinput"];

fn main() {
    for lib in LIBS {
        pkg_config::Config::new().probe(lib).unwrap();
    }
}
