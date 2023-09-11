fn main() {
    use std::{fs::OpenOptions, path::Path};

    // Only re-run if the Minecraft JSON data files were modified.
    //println!("cargo:rerun-if-changed=../../mc-data/blocks.json");

    let out_dir = std::env::var_os("OUT_DIR").expect("environment variable OUT_DIR set");
    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(Path::new(&out_dir).join("blocks.rs"))
        .expect("create destination file");

    let src_path = Path::new("../../contrib/blocks.json");
    let json_data = std::fs::read_to_string(&src_path).expect("read input JSON");

    rjacraft_data_generator::gen_structs(json_data, &mut dest_file).expect("generate code");
}
