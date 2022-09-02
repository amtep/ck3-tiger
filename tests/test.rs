use lazy_static::lazy_static;
use std::path::PathBuf;
use std::sync::Mutex;

use ck3_mod_validator::errors::{log_to, set_mod_root, set_vanilla_root, take_log_to};
use ck3_mod_validator::everything::Everything;

lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

fn check_mod_helper(modname: &str) -> String {
    let _guard = TEST_MUTEX.lock().unwrap();

    let vanilla_root = PathBuf::from("tests/files/ck3");
    let mod_root = PathBuf::from(format!("tests/files/{}", modname));

    set_vanilla_root(vanilla_root.clone());
    set_mod_root(mod_root.clone());
    log_to(Box::new(Vec::new()));

    let mut everything = Everything::new(&vanilla_root, &mod_root, Vec::new()).unwrap();
    everything.load_all();
    everything.validate_all();

    let errors = (*take_log_to()).get_logs().unwrap();
    eprint!("{}", &errors);
    errors
}

#[test]
fn test_mod_1() {
    let errors = check_mod_helper("mod1");

    // TODO: check for absence of duplicate event warning for non-dup.0001

    assert!(errors.contains("bad_loca_name.yml: ERROR: could not determine language from filename"));

    assert!(
        errors.contains("decision.txt:2:1: ERROR: missing english localization key my_decision\n")
    );
    assert!(errors
        .contains("decision.txt:2:1: ERROR: missing english localization key my_decision_tooltip"));
    assert!(errors
        .contains("decision.txt:2:1: ERROR: missing english localization key my_decision_desc"));
    assert!(errors
        .contains("decision.txt:2:1: ERROR: missing english localization key my_decision_confirm"));

    assert!(errors
        .contains("decision.txt:8:13: ERROR: missing english localization key my_decision_also"));
    assert!(errors.contains(
        "decision.txt:9:12: ERROR: missing english localization key my_decision2_description"
    ));
    assert!(errors
        .contains("decision.txt:10:25: ERROR: missing english localization key totally_different"));
    assert!(errors
        .contains("decision.txt:11:20: ERROR: missing english localization key my_decision2_c"));

    assert!(errors.contains("decision.txt:7:15: ERROR: referenced file does not exist"));
}

#[test]
fn test_mod_2() {
    let errors = check_mod_helper("mod2");

    assert!(errors.contains(
        "interaction.txt:1:1: ERROR: missing english localization key test_interaction\n"
    ));
    assert!(errors.contains(
        "interaction.txt:3:5: ERROR: missing english localization key test_interaction_extra_icon"
    ));
    assert!(errors.contains("interaction.txt:3:18: ERROR: referenced file does not exist"));
    assert!(errors.contains("interaction.txt:2:12: ERROR: file gfx/interface/icons/character_interactions/missing_icon.dds does not exist"));
}
