use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

use ck3_tiger::errors::{log_to, set_mod_root, set_vanilla_root};
use ck3_tiger::everything::Everything;

fn apw_simple(c: &mut Criterion) {
    let vanilla_root = PathBuf::from("/home/gamer/CK3");
    let mod_root = PathBuf::from("/home/gamer/Pdx/mod/A Primordial World");

    set_vanilla_root(vanilla_root.clone());
    set_mod_root(mod_root.clone());
    log_to(Box::new(Vec::new()));

    c.bench_function("apw", |b| {
        b.iter(|| {
            let mut everything = Everything::new(&vanilla_root, &mod_root, Vec::new()).unwrap();
            everything.load_all();
            everything.validate_all();
        })
    });
}

fn pod_simple(c: &mut Criterion) {
    let vanilla_root = PathBuf::from("/home/gamer/CK3");
    let mod_root = PathBuf::from("/home/dark/repo/github/pod/dev/devpod");

    set_vanilla_root(vanilla_root.clone());
    set_mod_root(mod_root.clone());
    log_to(Box::new(Vec::new()));

    c.bench_function("pod", |b| {
        b.iter(|| {
            let mut everything = Everything::new(&vanilla_root, &mod_root, Vec::new()).unwrap();
            everything.load_all();
            everything.validate_all();
        })
    });
}

criterion_group!(apw, apw_simple);
criterion_group!(pod, pod_simple);
criterion_main!(apw, pod);
