use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

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
    let mut hypervisor_cargo_args: Vec<String> = vec!["build".to_string()];
    let mut l1_hypervisor_cargo_args: Vec<String> = vec!["build".to_string()];
    let mut hypervisor_output_directory = project_root().join("bin/disk");
    let mut l1_hypervisor_output_directory = project_root().join("bin/L1disk");

    // Path
    let hypervisor_path = project_root().join("hypervisor");
    let l1_hypervisor_path = project_root().join("l1_hypervisor");

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

    let mut hypervisor_binary_path = project_root();
    let mut l1_hypervisor_binary_path = project_root();

    hypervisor_binary_path.push("target");
    hypervisor_binary_path.push(target);
    l1_hypervisor_binary_path.push("target");
    l1_hypervisor_binary_path.push(target);

    if is_release {
        hypervisor_binary_path.push("release");
        l1_hypervisor_binary_path.push("release");
        hypervisor_cargo_args.push("--release".to_string());
        l1_hypervisor_cargo_args.push("--release".to_string());
    } else {
        hypervisor_binary_path.push("debug");
        l1_hypervisor_binary_path.push("debug");
    }

    hypervisor_binary_path.push("hypervisor");
    l1_hypervisor_binary_path.push("l1_hypervisor");

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    if is_nested {
        hypervisor_cargo_args.push("--features".to_string());
        hypervisor_cargo_args.push("nested_support".to_string());
        let _ = Command::new(&cargo)
            .current_dir(l1_hypervisor_path)
            .args(&l1_hypervisor_cargo_args)
            .status()?;

        fs::create_dir_all(&l1_hypervisor_output_directory)?;
        l1_hypervisor_output_directory.push("hypervisor");

        fs::rename(l1_hypervisor_binary_path, l1_hypervisor_output_directory)?;
    }

    let _ = Command::new(&cargo)
        .current_dir(hypervisor_path)
        .args(hypervisor_cargo_args)
        .status()?;

    fs::create_dir_all(&hypervisor_output_directory)?;
    hypervisor_output_directory.push("hypervisor");

    fs::rename(hypervisor_binary_path, hypervisor_output_directory)?;

    Ok(())
}

fn run() -> Result<(), DynError> {
    // default settings
    let qemu = "qemu-system-riscv64".to_string();
    let mut is_debug = false;
    let smp = "1".to_string();
    let memory = "2G".to_string();

    // Path
    let hypervisor_directory = "bin/disk".to_string();
    let l1_hypervisor_directory = "bin/L1disk".to_string();
    let bios_binary_path = "bin/disk/u-boot".to_string();
    let vm_bios_binary_path = "bin/u-boot.bin".to_string();
    let vm_fdt_binary_path = "bin/virt.dtb".to_string();

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
        format!("file=fat:rw:{hypervisor_directory},format=raw,if=none,media=disk,id=drive0")
            .as_str(),
        "-device",
        "virtio-blk-device,drive=drive1,bus=virtio-mmio-bus.0",
        "-drive",
        format!("file={vm_bios_binary_path},format=raw,if=none,id=drive1").as_str(),
        "-device",
        "virtio-blk-device,drive=drive2,bus=virtio-mmio-bus.1",
        "-drive",
        format!("file={vm_fdt_binary_path},format=raw,if=none,id=drive2").as_str(),
        "-device",
        "virtio-blk-device,drive=drive3,bus=virtio-mmio-bus.2",
        "-drive",
        format!("file=fat:rw:{l1_hypervisor_directory},format=raw,if=none,id=drive3").as_str(),
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
