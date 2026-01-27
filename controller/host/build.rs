//! Build script that contains all the information to build the slint UI.

use std::io;

fn main() -> io::Result<()> {
    // Slint build
    let config = slint_build::CompilerConfiguration::new();
    slint_build::compile_with_config("ui/main.slint", config).expect("Slint build failed");

    Ok(())
}
