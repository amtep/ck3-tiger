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

    let mut everything = Everything::new(&vanilla_root, &mod_root).unwrap();
    everything.load_all();
    everything.check_all();

    let errors = (*take_log_to()).get_logs().unwrap();
    eprint!("{}", &errors);

    // TODO: check for absence of duplicate event warning for non-dup.0001

    assert!(errors.contains("bad_loca_name.yml: ERROR: could not determine language from filename"));

    assert!(
        errors.contains("decision.txt:2:12: ERROR: missing english localization key my_decision ")
    );
    assert!(errors.contains(
        "decision.txt:2:12: ERROR: missing english localization key my_decision_tooltip "
    ));
    assert!(errors
        .contains("decision.txt:2:12: ERROR: missing english localization key my_decision_desc "));
    assert!(errors.contains(
        "decision.txt:2:12: ERROR: missing english localization key my_decision_confirm "
    ));

    assert!(errors
        .contains("decision.txt:8:31: ERROR: missing english localization key my_decision_also "));
    assert!(errors.contains(
        "decision.txt:9:38: ERROR: missing english localization key my_decision2_description "
    ));
    assert!(errors.contains(
        "decision.txt:10:44: ERROR: missing english localization key totally_different "
    ));
    assert!(errors
        .contains("decision.txt:11:36: ERROR: missing english localization key my_decision2_c "));
}
