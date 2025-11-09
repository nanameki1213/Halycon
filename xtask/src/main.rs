use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
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

    l1_hypervisor_cargo_args.push("--target".to_string());
    l1_hypervisor_cargo_args.push(target.to_string());
    l1_hypervisor_cargo_args.push("--package".to_string());
    l1_hypervisor_cargo_args.push("l1_hypervisor".to_string());
    hypervisor_cargo_args.push("--target".to_string());
    hypervisor_cargo_args.push(target.to_string());

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    if is_nested {
        hypervisor_cargo_args.push("--features".to_string());
        hypervisor_cargo_args.push("nested_support".to_string());
        let _ = Command::new(&cargo)
            .current_dir(project_root())
            .args(&l1_hypervisor_cargo_args)
            .status()?;

        fs::create_dir_all(&l1_hypervisor_output_directory)?;
        l1_hypervisor_output_directory.push("hypervisor");
        
        fs::rename(
            l1_hypervisor_binary_path,
            l1_hypervisor_output_directory
        )?;
    }   
    
    let _ = Command::new(&cargo)
        .current_dir(project_root())
        .args(hypervisor_cargo_args)
        .status()?;

    fs::create_dir_all(&hypervisor_output_directory)?;
    hypervisor_output_directory.push("hypervisor");

    fs::rename(
        hypervisor_binary_path,
        hypervisor_output_directory
    )?;

    Ok(())
}

fn run() -> Result<(), DynError> {
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

Examples:
  cargo xtask build  Build Halycon with default option."
    )
}
