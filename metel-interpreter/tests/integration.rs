#[path = "integration/harness/mod.rs"]
mod harness;

macro_rules! register_integration_test {
    ($name:ident, $suite:literal, $path:literal) => {
        #[allow(non_snake_case)]
        #[test]
        fn $name() {
            crate::harness::run_discovered_fixture($suite, std::path::Path::new($path));
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/integration_generated.rs"));
