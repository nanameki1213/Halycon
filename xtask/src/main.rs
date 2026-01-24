pub mod tasks;

use simple_logger::SimpleLogger;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tasks::create_disk;
use tasks::device_tree;
use tasks::mkimage;

type DynError = Box<dyn std::error::Error>;

fn main() {
    SimpleLogger::new().init().unwrap();

    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("build") => build()?,
        Some("run") => run()?,
        Some("test") => test()?,
        _ => print_help(),
    }
    Ok(())
}

fn build() -> Result<(), DynError> {
    let target = "riscv64gc-unknown-none-elf";
    // Default settings
    let mut is_release = false;
    let mut cargo_features: Vec<String> = Vec::new();

    // Path
    let base_output_directory = project_root().join("bin");
    let hypervisor_output_directory = base_output_directory.clone().join("disk");
    let l1_hypervisor_output_directory = base_output_directory.clone().join("l1_disk");
    let l2_disk = base_output_directory.clone().join("l2_disk");
    let script_path = project_root().join("scripts");

    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter().skip(2);

    // Parse options
    while let Some(v) = args_iter.next() {
        if v == "-f" || v == "--features" {
            if let Some(feature_list) = args_iter.next() {
                for feature in feature_list.split(',') {
                    let feature_name = feature.trim();
                    match feature_name {
                        "nested" => cargo_features.push("nested_support".to_string()),
                        "nested_acel" => cargo_features.push("nested_acceleration".to_string()),
                        _ => cargo_features.push(feature_name.to_string()),
                    }
                }
            }
        } else if v == "-r" || v == "--release" {
            is_release = true;
        } else if v == "-h" || v == "--help" {
            print_help();
            return Ok(());
        }
    }

    fs::create_dir_all(&hypervisor_output_directory)?;

    let need_l1_build = cargo_features
        .iter()
        .any(|f| f == "nested_support" || f == "nested_acceleration");

    if need_l1_build {
        fs::create_dir_all(&l1_hypervisor_output_directory)?;
        fs::create_dir_all(&l2_disk)?;
        log::info!("build l1 hypervisor");
        let output_path = l1_hypervisor_output_directory.clone().join("hypervisor");
        let mut binary_path = project_root().join(format!("target/{}", target));
        if is_release {
            binary_path.push("release/l1_hypervisor");
        } else {
            binary_path.push("debug/l1_hypervisor");
        }

        let l1_cargo_features: Vec<String> = cargo_features
            .iter()
            .filter(|&f| f == "nested_acceleration")
            .cloned()
            .collect();

        build_l1_hypervisor(&l1_cargo_features, is_release)?;
        fs::rename(&binary_path, &output_path)?;

        // compile device tree script
        log::info!("compile device tree script for L1 Hypervisor");
        let dts_path = script_path.clone().join("virt_L1hypervisor.dts");
        let output_path = l1_hypervisor_output_directory.clone().join("virt.dtb");
        device_tree::compile_dts(&dts_path, &output_path)?;

        // compile boot script
        log::info!("compile u-boot boot script for L2 VM");
        let binary_path = script_path.clone().join("boot_L2hypervisor.script");
        let output_path = l2_disk.clone().join("boot.scr");
        mkimage::uboot_mkimage(&binary_path, &output_path)?;

        let entries = read_dir_entries(&l2_disk)?;
        let files: Vec<&Path> = entries.iter().map(|e| e.as_path()).collect();
        create_disk::create_fat32_disk(
            &l1_hypervisor_output_directory.clone().join("vm.img"),
            &files,
        )?;

        // compile boot script
        log::info!("compile u-boot boot script for L1 Hypervisor");
        let binary_path = script_path.clone().join("boot_L1hypervisor.script");
        let output_path = l1_hypervisor_output_directory.clone().join("boot.scr");
        mkimage::uboot_mkimage(&binary_path, &output_path)?;

        // create image
        let entries = read_dir_entries(&l1_hypervisor_output_directory)?;
        let files: Vec<&Path> = entries.iter().map(|e| e.as_path()).collect();

        log::info!("create disk");
        create_disk::create_fat32_disk(
            &hypervisor_output_directory.clone().join("vm.img"),
            &files,
        )?;
    }

    // build hypervisor
    log::info!("build hypervisor");
    let output_path = hypervisor_output_directory.clone().join("hypervisor");
    let mut binary_path = project_root().join(format!("target/{}", target));
    if is_release {
        binary_path.push("release/hypervisor");
    } else {
        binary_path.push("debug/hypervisor");
    }

    build_hypervisor(&cargo_features, is_release)?;

    fs::rename(&binary_path, &output_path)?;

    // compile device tree script
    log::info!("compile device tree script");
    let dts_path = script_path.clone().join("virt.dts");
    let output_path = hypervisor_output_directory.clone().join("virt.dtb");
    device_tree::compile_dts(&dts_path, &output_path)?;

    // compile boot script
    log::info!("compile u-boot boot script");
    let binary_path = script_path.clone().join("boot.script");
    let output_path = hypervisor_output_directory.clone().join("boot.scr");
    mkimage::uboot_mkimage(&binary_path, &output_path)?;

    // create image
    // make list of file in `hypervisor_output_directory`
    let entries = read_dir_entries(&hypervisor_output_directory)?;
    let files: Vec<&Path> = entries.iter().map(|e| e.as_path()).collect();

    log::info!("create disk");
    create_disk::create_fat32_disk(&project_root().join("disk.img"), &files)?;

    Ok(())
}

fn build_hypervisor(features: &[String], is_release: bool) -> Result<(), DynError> {
    let hypervisor_path = project_root().join("hypervisor");
    let mut hypervisor_cargo_args: Vec<String> = vec!["build".to_string()];

    if is_release {
        hypervisor_cargo_args.push("--release".to_string());
    }

    if !features.is_empty() {
        hypervisor_cargo_args.push("--features".to_string());
        hypervisor_cargo_args.push(features.join(","));
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let _ = Command::new(&cargo)
        .current_dir(hypervisor_path)
        .args(hypervisor_cargo_args)
        .status()?;

    Ok(())
}

fn build_l1_hypervisor(features: &[String], is_release: bool) -> Result<(), DynError> {
    let l1_hypervisor_path = project_root().join("l1_hypervisor");
    let mut l1_hypervisor_cargo_args: Vec<String> = vec!["build".to_string()];

    if is_release {
        l1_hypervisor_cargo_args.push("--release".to_string());
    }

    if !features.is_empty() {
        l1_hypervisor_cargo_args.push("--features".to_string());
        l1_hypervisor_cargo_args.push(features.join(","));
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let _ = Command::new(&cargo)
        .current_dir(l1_hypervisor_path)
        .args(l1_hypervisor_cargo_args)
        .status()?;

    Ok(())
}

fn read_dir_entries(path: &Path) -> Result<Vec<PathBuf>, DynError> {
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    Ok(entries)
}

fn run() -> Result<(), DynError> {
    // default settings
    let qemu = "qemu-system-riscv64".to_string();
    let mut is_debug = false;
    let smp = "1".to_string();
    let memory = "2G".to_string();

    // BIOS path
    let bios_binary_path = "bin/u-boot".to_string();
    // Disk image path
    let host_disk_image_path = "disk.img".to_string();

    // make image for host and vm

    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter().skip(2);

    // Parse options
    while let Some(v) = args_iter.next() {
        if v == "--debug" {
            is_debug = true;
        }
    }

    let mut qemu_command = Command::new(qemu);
    qemu_command.args([
        "-M",
        "virt",
        "-smp",
        smp.as_str(),
        "-bios",
        bios_binary_path.as_str(),
        "-nographic",
        "-m",
        memory.as_str(),
        "-device",
        "virtio-blk-device,drive=drive0,bus=virtio-mmio-bus.0",
        "-drive",
        format!("file={host_disk_image_path},format=raw,if=none,media=disk,id=drive0").as_str(),
        "-global",
        "virtio-mmio.force-legacy=false",
        "-D",
        "logfile.log",
        "-d",
        "in_asm,int",
    ]);
    if is_debug {
        qemu_command.args(["-s", "-S"]);
    }

    qemu_command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to run qemu");

    Ok(())
}

fn test() -> Result<(), DynError> {
    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter().skip(2);
    let mut target_package = None;

    // Parse options
    while let Some(v) = args_iter.next() {
        if v == "--package" {
            if let Some(package) = args_iter.next() {
                target_package = Some(package);
            } else {
                return Err("Error: Package name required after --package".into());
            }
        }
    }

    let should_prepare_disk = match target_package.map(|s| s.as_str()) {
        Some("fat32") => true,
        None => true,
        _ => false,
    };

    if should_prepare_disk {
        log::info!("Preparing FAT32 disk image for testing...");
        create_test_image()?;
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(&cargo);

    cmd.current_dir(project_root());
    cmd.arg("test");

    if let Some(pkg) = target_package {
        log::info!("Running tests for package: {}", pkg);
        cmd.arg("--package").arg(pkg);
    } else {
        log::info!("Running tests for all workspace members");
        cmd.arg("--workspace");
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err("Tests failed".into());
    }

    Ok(())
}

fn create_test_image() -> Result<(), DynError> {
    let tmp_dir = project_root().join("target/tmp_fat32_test");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir_all(&tmp_dir)?;

    let mut files_paths = Vec::new();

    for i in 1..5 {
        let file_name = format!("TEST{}.TXT", i);
        let file_path = tmp_dir.join(&file_name);
        let content = format!(
            "The process of analyzing a FAT32 file system using a hex editor requires a deep understanding of how data is structured across sectors and clusters. This specific paragraph is designed to exceed the standard sector size of 512 bytes, ensuring that your read test can verify whether the file system driver or your manual parsing logic correctly handles data that spans across multiple sectors. When you examine this file in a hex dump, you should notice that the text continues past the first 0x200 bytes offset. If the file is stored in cluster 2, for example, you can calculate its physical location by identifying the start of the data region. Remember that in FAT32, the root directory is no longer at a fixed location but is treated as a cluster chain. This provides more flexibility compared to older FAT versions. By reading this entire passage successfully, you confirm that your environment can handle basic file I/O operations and that your offset calculations from the MBR to the BPB, and finally to the data area, are accurate."
        );

        fs::write(&file_path, content)?;
        files_paths.push(file_path);
    }

    let file_refs: Vec<&Path> = files_paths.iter().map(|p| p.as_path()).collect();

    let output_img = project_root().join("test_disk.img");

    create_disk::create_fat32_disk(&output_img, &file_refs)?;

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn print_help() {
    eprintln!(
        "Usage: cargo xtask command [OPTION]

Command List:
  build
  run

  -f, --features [nested, nested_acel, ...]
                            select options with comma split
Examples:
  cargo xtask build              Build Halycon with default option.
  cargo xtask build -f nested    Build Halycon with nested_support.
  cargo xtask build -f nested_acel    Build Halycon with nested_acceleration."
    )
}
