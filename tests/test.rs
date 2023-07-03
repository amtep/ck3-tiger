use lazy_static::lazy_static;
use std::path::PathBuf;
use std::sync::Mutex;

use ck3_tiger::errors::{log_to, set_mod_root, set_vanilla_dir, take_log_to};
use ck3_tiger::everything::Everything;

lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

fn check_mod_helper(modname: &str) -> String {
    let _guard = TEST_MUTEX.lock().unwrap();

    let vanilla_dir = PathBuf::from("tests/files/ck3");
    let mod_root = PathBuf::from(format!("tests/files/{}", modname));

    set_vanilla_dir(vanilla_dir.clone());
    set_mod_root(mod_root.clone());
    log_to(Box::new(Vec::new()));

    let mut everything = Everything::new(&vanilla_dir, &mod_root, Vec::new()).unwrap();
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

    assert!(errors.contains(
        "[MOD] file localization/english/bad_loca_name.yml
ERROR (filename): could not determine language from filename"
    ));

    assert!(errors
        .contains("ERROR (missing-localization): missing english localization key my_decision\n"));
    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key my_decision_tooltip"
    ));
    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key my_decision_desc"
    ));
    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key my_decision_confirm"
    ));

    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key my_decision_also"
    ));
    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key my_decision2_description"
    ));
    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key totally_different"
    ));
    assert!(errors
        .contains("ERROR (missing-localization): missing english localization key my_decision2_c"));

    assert!(errors.contains(
        "[MOD] file common/decisions/decision.txt
line 3     picture = \"\"
line 3               ^
ERROR (missing-file): referenced file does not exist"
    ));
}

#[test]
fn test_mod_2() {
    let errors = check_mod_helper("mod2");

    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key test_interaction"
    ));
    assert!(errors.contains(
        "ERROR (missing-localization): missing english localization key test_interaction_extra_icon"
    ));
    assert!(errors.contains(
        "[MOD] file common/character_interactions/interaction.txt
line 3     extra_icon = \"gfx/also_missing\"
line 3                  ^
ERROR (missing-file): referenced file does not exist"
    ));
    assert!(errors.contains("ERROR (missing-file): file gfx/interface/icons/character_interactions/missing_icon.dds does not exist"));
}
