use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tiger_lib::{Everything, ModFile};

static CONFIG_PATH: &str = "./benches/config.toml";

// Sample Config File:
// vanilla_dir = "..."
// modfile_dir = "..."
// modfile_paths = ["...", "..."]
// sample_size = 30

#[derive(Deserialize)]
struct Config {
    vanilla_dir: String,
    modfile_dir: Option<String>,
    modfile_paths: Vec<String>,
    sample_size: Option<usize>,
}

fn bench_multiple(c: &mut Criterion) {
    let content = fs::read_to_string(CONFIG_PATH).unwrap();
    let config: Config = toml::from_str(&content).unwrap();
    let mut modfile_paths = config.modfile_paths.iter().map(PathBuf::from).collect::<Vec<_>>();

    if let Some(modfile_dir) = config.modfile_dir {
        let iter =
            fs::read_dir(modfile_dir).unwrap().filter_map(|entry| entry.ok()).filter_map(|entry| {
                entry.file_name().to_string_lossy().ends_with(".mod").then(|| entry.path())
            });
        modfile_paths.extend(iter);
    }

    let mut group = c.benchmark_group("benchmark");
    group.sample_size(config.sample_size.unwrap_or(10));
    for (index, modfile_path) in modfile_paths.iter().enumerate() {
        let modfile = ModFile::read(modfile_path).unwrap();
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

fn bench_mod(vanilla_dir: &str, modfile: &ModFile) {
    let mut everything =
        Everything::new(Some(Path::new(vanilla_dir)), &modfile.modpath(), modfile.replace_paths())
            .unwrap();
    everything.load_all();
    everything.validate_all();
}

criterion_group!(benches, bench_multiple);
criterion_main!(benches);
