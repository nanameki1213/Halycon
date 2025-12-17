use std::{path::Path, process::Command};

use crate::project_root;

type DynError = Box<dyn std::error::Error>;

pub fn compile_dts(src: &Path, dst: &Path) -> Result<(), DynError> {
    let dtc_path = project_root().join("u-boot/u-boot/scripts/dtc/dtc");

    // TODO: When dst path is pointing directory, return error.
    Command::new(&dtc_path).args([
        "-I",
        "dts",
        "-O",
        "dtb",
        "-o",
        dst.to_str().unwrap(),
        src.to_str().unwrap(),
    ]).status()?;

    Ok(())
}
