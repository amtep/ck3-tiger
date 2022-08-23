use std::path::PathBuf;

use ck3_mod_validator::errors::{log_to, set_mod_root, set_vanilla_root, take_log_to};
use ck3_mod_validator::everything::Everything;

#[test]
fn test_mod_1() {
    let vanilla_root = PathBuf::from("tests/files/ck3");
    let mod_root = PathBuf::from("tests/files/mod1");

    set_vanilla_root(vanilla_root.clone());
    set_mod_root(mod_root.clone());
    log_to(Box::new(Vec::new()));

    let mut everything = Everything::new(vanilla_root, mod_root).unwrap();
    everything.load_all();
    everything.check_all();

    let errors = (*take_log_to()).get_logs().unwrap();
    eprint!("{}", &errors);

    // TODO: check for absence of duplicate event warning for non-dup.0001

    assert!(errors.contains("bad_loca_name.yml: ERROR: could not determine language from filename"));
}
