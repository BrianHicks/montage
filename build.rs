fn main() {
    println!("cargo:rerun-if-changed=migrations");

    cynic_codegen::register_schema("montage")
        .from_sdl_file("src/client/schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
