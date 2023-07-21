fn main() {
    cynic_codegen::register_schema("montage")
        .from_sdl_file("schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
