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
        _ => print_help(),
    }
    Ok(())
}

fn build() -> Result<(), DynError> {
    let target = "riscv64gc-unknown-none-elf";
    // Default settings
    let mut is_release = false;
    let mut is_nested = false;

    // Path
    let base_output_directory = project_root().join("bin");
    let hypervisor_output_directory = base_output_directory.clone().join("disk");
    // let l1_hypervisor_path = project_root().join("l1_hypervisor");
    // let hypervisor_image_path = project_root().join("disk.img");
    // let vm_disk_image_path = project_root().join("vm.img");
    let script_path = project_root().join("scripts");

    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter().skip(2);
    // Parse options
    while let Some(v) = args_iter.next() {
        if v == "-f" || v == "--features" {
            if let Some(feature_list) = args_iter.next() {
                for feature in feature_list.split(',') {
                    let feature_name = feature.trim().to_string();
                    if feature_name == "nested" {
                        is_nested = true;
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

    // build hypervisor
    log::info!("build hypervisor");
    let output_path = hypervisor_output_directory.clone().join("hypervisor");
    let mut binary_path = project_root().join(format!("target/{}", target));
    if is_release {
        binary_path.push("release/hypervisor");
    } else {
        binary_path.push("debug/hypervisor");
    }
    build_hypervisor(is_nested, is_release)?;
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

    if is_nested {
        log::info!("build l1 hypervisor");
        build_l1_hypervisor(is_release)?;
    }

    // Create image
    // make list of file in `hypervisor_output_directory`
    let entries = read_dir_entries(&hypervisor_output_directory)?;
    let files: Vec<&Path> = entries.iter().map(|e| e.as_path()).collect();
    log::info!("files: {:?}", files);

    log::info!("create disk");
    create_disk::create_fat32_disk(&project_root().join("disk.img"), &files)?;

    Ok(())
}

fn build_hypervisor(is_nested: bool, is_release: bool) -> Result<(), DynError> {
    let hypervisor_path = project_root().join("hypervisor");
    let mut hypervisor_cargo_args: Vec<String> = vec!["build".to_string()];

    if is_release {
        hypervisor_cargo_args.push("--release".to_string());
    }
    if is_nested {
        hypervisor_cargo_args.push("--features".to_string());
        hypervisor_cargo_args.push("nested_support".to_string());
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let _ = Command::new(&cargo)
        .current_dir(hypervisor_path)
        .args(hypervisor_cargo_args)
        .status()?;

    Ok(())
}

fn build_l1_hypervisor(is_release: bool) -> Result<(), DynError> {
    let l1_hypervisor_path = project_root().join("l1_hypervisor");
    let mut l1_hypervisor_cargo_args: Vec<String> = vec!["build".to_string()];

    if is_release {
        l1_hypervisor_cargo_args.push("--release".to_string());
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
    let vm_disk_image_path = "vm.img".to_string();

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
        "virtio-blk-device,drive=drive0",
        "-drive",
        format!("file={host_disk_image_path},format=raw,if=none,media=disk,id=drive0").as_str(),
        "-device",
        "virtio-blk-device,drive=drive1,bus=virtio-mmio-bus.0",
        "-drive",
        format!("file={vm_disk_image_path},format=raw,if=none,media=disk,id=drive1").as_str(),
        "-device",
        "virtio-blk-device,drive=drive2,bus=virtio-mmio-bus.1",
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

  -f, --features [nested]
                           select options with comma split
Examples:
  cargo xtask build             Build Halycon with default option.
  cargo xtask build -f nested   Build Halycon with nested support hypervisor and l1_hypervisor."
    )
}
