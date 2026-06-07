use std::path::Path;

use super::fixture::resolve_fixture_config;
use super::runners::run_fixture;

pub fn run_discovered_fixture(suite: &str, relative_path: &Path) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let config = resolve_fixture_config(suite, &path);
    run_fixture(&path, &config);
}
