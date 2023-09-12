use std::{fs::File, path::Path};

fn main() {
    // Only re-run if the Minecraft JSON data files were modified.
    //println!("cargo:rerun-if-changed=../../mc-data/blocks.json");

    let out_dir = std::env::var_os("OUT_DIR").expect("env variable OUT_DIR set");
    let dest_path = Path::new(&out_dir).join("blocks.rs");
    let mut dest = File::create(dest_path).expect("create destination file for blocks");

    let blocks =
        std::env::var_os("RJA_DATA_BLOCKS").unwrap_or_else(|| "../../contrib/blocks.json".into());
    let mut blocks = File::open(&blocks).expect("open JSON file with blocks");

    rjacraft_data_generator::gen_structs(&mut blocks, &mut dest).expect("generate code");
}
