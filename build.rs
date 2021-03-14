fn main() {
    tonic_build::configure()
        .out_dir("src/fixtures")
        .compile(&["examples/hello.proto"], &["examples"])
        .unwrap();
}
