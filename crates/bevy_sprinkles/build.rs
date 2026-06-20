use std::{env, fs, path::Path};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let flipbooks_dir =
        Path::new(&manifest).join("src/textures/assets/brackeys_vfx_bundle/flipbooks");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    println!("cargo:rerun-if-changed={}", flipbooks_dir.display());

    let mut entries: Vec<(String, String, u32, u32)> = Vec::new();

    if let Ok(dir) = fs::read_dir(&flipbooks_dir) {
        let mut paths: Vec<_> = dir
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("png") || ext.eq_ignore_ascii_case("tga"))
                    .unwrap_or(false)
            })
            .collect();
        paths.sort_by_key(|e| e.file_name());

        for entry in paths {
            let path = entry.path();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            println!("cargo:rerun-if-changed={}", path.display());
            let display_name = stem_to_display_name(&stem);
            let (columns, rows) = parse_grid_size(&stem);
            let relative_path = format!("assets/brackeys_vfx_bundle/flipbooks/{filename}");
            entries.push((display_name, relative_path, columns, rows));
        }
    }

    // flipbook_list.rs: pub const FLIPBOOK_TEXTURES: &[(&str, &str, u32, u32)]
    let mut list_code = String::from(
        "/// Flipbook sprite-sheet textures from the Brackeys VFX Bundle, as `(display_name, embedded_path, columns, rows)` tuples.\npub const FLIPBOOK_TEXTURES: &[(&str, &str, u32, u32)] = &[\n",
    );
    for (display, path, columns, rows) in &entries {
        let embedded = format!("embedded://bevy_sprinkles/textures/{path}");
        list_code.push_str(&format!(
            "    (\"{display}\", \"{embedded}\", {columns}, {rows}),\n"
        ));
    }
    list_code.push_str("];\n");
    fs::write(out_path.join("flipbook_list.rs"), &list_code).unwrap();

    // flipbook_embeds.rs: one explicit embedded registry insert per file.
    // This file is generated in OUT_DIR, so embedded_asset! would resolve
    // include_bytes! paths relative to OUT_DIR instead of this crate's src.
    let mut embed_code = String::from(
        "{\n    let embedded = app.world_mut().resource_mut::<bevy::asset::io::embedded::EmbeddedAssetRegistry>();\n",
    );
    for (_, path, _, _) in &entries {
        let source_path = format!("src/textures/{path}");
        let embedded_path = format!("bevy_sprinkles/textures/{path}");
        embed_code.push_str(&format!(
            "    embedded.insert_asset(\n        std::path::PathBuf::from(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{source_path}\")),\n        std::path::Path::new(\"{embedded_path}\"),\n        include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{source_path}\")),\n    );\n"
        ));
    }
    embed_code.push_str("}\n");
    fs::write(out_path.join("flipbook_embeds.rs"), &embed_code).unwrap();
}

fn stem_to_display_name(stem: &str) -> String {
    stem.split(['_', '-'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_grid_size(stem: &str) -> (u32, u32) {
    let Some(grid) = stem.rsplit_once('_').map(|(_, grid)| grid) else {
        return (1, 1);
    };
    let Some((columns, rows)) = grid.split_once('x') else {
        return (1, 1);
    };

    match (columns.parse(), rows.parse()) {
        (Ok(columns), Ok(rows)) if columns > 0 && rows > 0 => (columns, rows),
        _ => (1, 1),
    }
}
