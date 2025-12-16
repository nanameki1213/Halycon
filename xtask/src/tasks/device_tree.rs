use std::{path::Path, process::Command};

use crate::project_root;

type DynError = Box<dyn std::error::Error>;

pub fn compile_dts(src: &Path, dst: &Path) -> Result<(), DynError> {
    let dtc_path = project_root().join("u-boot/u-boot/scripts/dtc/dtc");

    Command::new(dtc_path.to_str().unwrap())
        .arg("-I")
        .arg("dts")
        .arg("-O")
        .arg("dtb")
        .arg("-o")
        .arg(src.to_str().unwrap())
        .arg(format!("{}/virt.dts", dst.to_str().unwrap()).as_str())
        .status()?;

    Ok(())
}
