use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=proto/");

    // Collect all .proto files in the proto directory
    let proto_files: Vec<PathBuf> = WalkDir::new("proto")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "proto"))
        .map(|e| e.path().to_owned())
        .collect();

    // Convert PathBuf to string references for the protobuf compiler
    let proto_paths: Vec<&str> = proto_files
        .iter()
        .filter_map(|path| path.to_str())
        .collect();

    // Print the files being compiled for debugging
    println!("Compiling proto files:");
    for path in &proto_paths {
        println!("  {}", path);
    }

    // Generate the Rust code
    protobuf_codegen::Codegen::new()
        .out_dir("src/proto")
        .inputs(&proto_paths)
        .include("proto")
        .run()
        .expect("Protobuf codegen failed.");
}