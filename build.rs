fn main() -> Result<(), Box<dyn std::error::Error>> {
    capnpc::CompilerCommand::new()
        .file("interface.capnp")
        .run()?;
    Ok(())
}
