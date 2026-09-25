fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(&["../../pulse.proto"], &["../.."])?;
    println!("cargo:rerun-if-changed=../../pulse.proto");
    Ok(())
}
