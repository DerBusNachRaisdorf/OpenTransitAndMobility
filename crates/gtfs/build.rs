fn main() -> std::io::Result<()> {
    let mut protobuf_out = std::path::PathBuf::new();
    protobuf_out.push(&std::env::var("OUT_DIR").unwrap());
    protobuf_out.push(&"protobuf");
    std::fs::create_dir(&protobuf_out).ok();
    prost_build::Config::new()
        .out_dir(&protobuf_out)
        //.default_package_filename("mod")
        .compile_protos(&["protobuf/gtfs-realtime.proto"], &["protobuf/"])?;
    Ok(())
}
