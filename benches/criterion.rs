use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tiger_lib::{Everything, ModFile};

static CONFIG_PATH: &str = "./benches/config.toml";

// Sample Config File:
// vanilla_dir = "path"
// modpaths = ["mod_path_1", "mod_path_2"]
// sample_size = 50

#[derive(Deserialize)]
struct Config {
    vanilla_dir: String,
    modpaths: Vec<String>,
    sample_size: Option<usize>,
}

fn bench_multiple(c: &mut Criterion) {
    let content = fs::read_to_string(CONFIG_PATH).unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    let mut group = c.benchmark_group("benchmark");
    group.sample_size(config.sample_size.unwrap_or(30));
    for (index, modpath) in config.modpaths.iter().enumerate() {
        let mut modpath = PathBuf::from(modpath);
        modpath.push("descriptor.mod");
        let modfile = ModFile::read(&modpath).unwrap();
        group.bench_with_input(
            BenchmarkId::new(
                "mods",
                format!("{}. {}", index + 1, modfile.display_name().unwrap_or_default()),
            ),
            &modfile,
            |b, modfile_ref| {
                b.iter(|| bench_mod(&config.vanilla_dir, modfile_ref));
            },
        );
    }

    group.finish();
}

fn bench_mod(vanilla_dir: &str, modfile: &ModFile) -> Everything {
    let mut everything =
        Everything::new(Some(Path::new(vanilla_dir)), &modfile.modpath(), modfile.replace_paths())
            .unwrap();
    everything.load_all();
    everything.validate_all();
    everything
}

criterion_group!(benches, bench_multiple);
criterion_main!(benches);
