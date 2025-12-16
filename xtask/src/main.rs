pub mod tasks;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tasks::create_disk;
use tasks::device_tree;

type DynError = Box<dyn std::error::Error>;

fn main() {
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
    let output_directory = project_root().join("bin");
    let l1_hypervisor_path = project_root().join("l1_hypervisor");
    let hypervisor_image_path = project_root().join("disk.img");
    let vm_disk_image_path = project_root().join("vm.img");

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

    // build hypervisor
    let hypervisor_binary_path = {
        let binary_path = project_root().join(format!("target/{}", target));
        if is_release {
            binary_path.push("release");
        }
        build_hypervisor(is_nested, is_release)?;
        binary_path
    };

    // compile device tree script
    let device_tree_script =
    device_tree::compile_dts(src, dst);

    if is_release {
        hypervisor_binary_path.push("release");
        l1_hypervisor_binary_path.push("release");
    } else {
        hypervisor_binary_path.push("debug");
        l1_hypervisor_binary_path.push("debug");
    }

    hypervisor_binary_path.push("hypervisor");
    l1_hypervisor_binary_path.push("l1_hypervisor");

    fs::create_dir_all(&output_directory)?;

    if is_nested {
        build_l1_hypervisor(is_release)?;       
        fs::rename(l1_hypervisor_binary_path, output_directory)?;
    }
    build_hypervisor(is_nested, is_release)?;

    fs::create_dir_all(&hypervisor_output_directory)?;
    hypervisor_output_directory.push("hypervisor");

    fs::rename(hypervisor_binary_path, hypervisor_output_directory)?;

    // Create image

    create_disk::create_fat32_disk(&hypervisor_image_path, &[
        &
    ])

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

fn run() -> Result<(), DynError> {
    // default settings
    let qemu = "qemu-system-riscv64".to_string();
    let mut is_debug = false;
    let smp = "1".to_string();
    let memory = "2G".to_string();

    // BIOS path
    let bios_binary_path = "bin/disk/u-boot".to_string();
    // Disk image path
    let host_disk_image_path = "host.disk".to_string();
    let vm_disk_image_path = "vm.disk".to_string();

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
