use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde::Deserialize;
use serde_json;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tiger_lib::{Everything, Game};

static CONFIG_PATH: &str = "./benches/config.toml";

// Sample Config File:
// vanilla_dir = "..."
// mod_dir = "..."
// mod_paths = ["...", "..."]
// sample_size = 30

#[derive(Deserialize)]
struct Config {
    vanilla_dir: String,
    mod_dir: Option<String>,
    mod_paths: Vec<String>,
    sample_size: Option<usize>,
}

fn bench_multiple(c: &mut Criterion) {
    Game::set(Game::Vic3).unwrap();
    let content = fs::read_to_string(CONFIG_PATH).unwrap();
    let config: Config = toml::from_str(&content).unwrap();
    let mut mod_paths = config.mod_paths.iter().map(PathBuf::from).collect::<Vec<_>>();

    if let Some(mod_dir) = config.mod_dir {
        let iter =
            fs::read_dir(mod_dir).unwrap().filter_map(|entry| entry.ok()).filter_map(|entry| {
                entry.path().join(".metadata/metadata.json").is_file().then(|| entry.path())
            });
        mod_paths.extend(iter);
    }

    let mut group = c.benchmark_group("benchmark");
    group.sample_size(config.sample_size.unwrap_or(10));
    for (index, mod_path) in mod_paths.iter().enumerate() {
        let metadata_file = mod_path.join(".metadata/metadata.json");
        let mod_data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(metadata_file).unwrap()).unwrap();
        group.bench_with_input(
            BenchmarkId::new("mods", format!("{}. {}", index + 1, mod_data["name"])),
            &mod_path,
            |b, modpath_ref| {
                b.iter(|| bench_mod(&config.vanilla_dir, modpath_ref));
            },
        );
    }

    group.finish();
}

fn bench_mod(vanilla_dir: &str, modpath: &Path) {
    let mut everything =
        Everything::new(None, Some(Path::new(vanilla_dir)), modpath, vec![]).unwrap();
    everything.load_all();
    everything.validate_all();
    everything.check_rivers();
}

criterion_group!(benches, bench_multiple);
criterion_main!(benches);
