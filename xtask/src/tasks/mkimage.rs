use std::{path::Path, process::Command};

use crate::{DynError, project_root};

pub fn uboot_mkimage(src: &Path, dst: &Path) -> Result<(), DynError> {
    let mkimage_path = project_root().join("u-boot/u-boot/tools/mkimage");

    // TODO: When dst path is pointing directory, return error.
    Command::new(&mkimage_path)
        .args([
            "-c",
            "none",
            "-A",
            "riscv",
            "-T",
            "script",
            "-d",
            src.to_str().unwrap(),
            dst.to_str().unwrap(),
        ])
        .status()?;

    Ok(())
}
