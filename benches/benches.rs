use criterion::*;
use dhtsearch::api::*;
use dhtsearch::file_types;

fn bench_file_types(c: &mut Criterion) -> anyhow::Result<()> {
    let json_payload_str = include_str!("3670d38c31d660d690384731483e145695586797.infoFiles.json");
    let payload: InfoFilesPayload = serde_json::from_str(json_payload_str)?;
    let info_files = &payload[0];
    c.bench_function("wat", |b| b.iter(|| file_types(info_files)));
    Ok(())
}

criterion_group!(benches, bench_file_types);
criterion_main!(benches);
