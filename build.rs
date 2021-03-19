use std::option_env;

fn main() {
    let build_enabled = option_env!("BUILD_PROTO")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !build_enabled {
        println!("=== Skipped compiling protos ===");
        return;
    }

    println!("Build...");
    tonic_build::configure()
        .out_dir("src/fixtures")
        .compile(&["examples/hello.proto"], &["examples"])
        .unwrap();
}
