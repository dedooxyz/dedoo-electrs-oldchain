fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)  // We only need the server
        .compile(
            &["proto/lightwallet.proto"],
            &["proto"],
        )?;
    Ok(())
}
