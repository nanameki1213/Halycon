use std::{
    fs, io::Write, path::{Path, PathBuf}, process::{Command, Stdio}
};

type DynError = Box<dyn std::error::Error>;

pub fn create_disk(image_path: &str, contents_dir: &str) -> Result<(), DynError> {
    let path = Path::new(image_path);
    
    if !path.exists() {
        Command::new("dd")
            .arg("if=/dev/zero")
            .arg(format!("of={image_path}").as_str())
            .arg("bs=1M")
            .arg("count=128")
            .output()?;
    }

    create_partition_with_fdisk(image_path)?;

    let output = Command::new("mkfs.fat")
        .arg("-F")
        .arg("32")
        .arg("--offset=2048")
        .arg(format!("{image_path}").as_str())
        .output()?;

    if !output.status.success() {
        return Err(format!(
                "Failed to run mkfs.fat. exit code: {}",
                String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    // make mount directory
    
    // mount directory
    
    // cp

    // unmount and delete mount directory

    Ok(())
}

fn create_partition_with_fdisk(image_path: &str) -> Result<(), DynError> {
    let fdisk_intaraction = [
        "o", // Create a new empty DOS partition table
        "n", // Add a new partition,
        "p", // Primary partition
        "1", // Partition number 1
        "2048", // First sector
        "", // Last sector (Default)
        "t", // Partition type
        "c", // Select FAT32
        "w", // Write table disk and exit
    ];

    let input_data = fdisk_intaraction.join("\n");

    let mut fdisk = Command::new("fdisk")
        .arg(image_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = fdisk.stdin.take() {
        stdin.write_all(input_data.as_bytes())?;
    }

    let status = fdisk.wait()?;

    if !status.success() {
        return Err(format!("Failed to run fdisk. exit code: {:?}", status.code()).into());
    }

    Ok(())
}
