use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Backend build and development tasks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    BuildFrontend,
    Build,
    Run,
    RunWithBuild,
    Fresh,
    Refresh,
    GenTs,
    GenEntity,
    GenLicense,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::BuildFrontend => build_frontend()?,
        Commands::Build => build()?,
        Commands::Run => run()?,
        Commands::RunWithBuild => run_with_build()?,
        Commands::Fresh => fresh()?,
        Commands::Refresh => refresh()?,
        Commands::GenTs => gen_ts()?,
        Commands::GenEntity => gen_entity()?,
        Commands::GenLicense => gen_license()?,
    }

    Ok(())
}

fn build_frontend() -> Result<(), Box<dyn std::error::Error>> {
    run_command("pnpm", &["install"], Some("../frontend"))?;
    run_command("pnpm", &["build"], Some("../frontend"))?;
    Ok(())
}

fn build() -> Result<(), Box<dyn std::error::Error>> {
    build_frontend()?;
    run_command("cargo", &["build", "--release"], None)?;
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--features", "dev"]);
    cmd.env("RUST_BACKTRACE", "1");
    cmd.env("RUST_LOG", "trace");
    cmd.env("BIND_ADDR", "127.0.0.1:8001");
    run_cmd(cmd)?;
    Ok(())
}

fn run_with_build() -> Result<(), Box<dyn std::error::Error>> {
    build_frontend()?;
    run()?;
    Ok(())
}

fn fresh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.arg("--");
    cmd.arg("fresh");
    cmd.env("DATA_PATH", "..");
    run_cmd_in_dir(cmd, "migration")?;
    Ok(())
}

fn refresh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.arg("--");
    cmd.arg("refresh");
    cmd.env("DATA_PATH", "..");
    run_cmd_in_dir(cmd, "migration")?;
    Ok(())
}

fn gen_ts() -> Result<(), Box<dyn std::error::Error>> {
    run_command(
        "typeshare",
        &[
            ".",
            "--lang=typescript",
            "--output-file=../frontend/src/lib/api/types.ts",
        ],
        None,
    )?;
    run_command("pnpm", &["i"], Some("../frontend"))?;
    run_command("pnpm", &["format-ffi"], Some("../frontend"))?;
    Ok(())
}

fn gen_entity() -> Result<(), Box<dyn std::error::Error>> {
    let sea_orm = which::which("sea-orm-cli")
        .map_err(|_| "sea-orm-cli not found. Please install it with: cargo install sea-orm-cli")?;

    run_command(
        sea_orm.to_str().unwrap(),
        &[
            "generate",
            "entity",
            "-u",
            "sqlite://db.sqlite",
            "-o",
            "entity/src/entities",
        ],
        None,
    )?;
    Ok(())
}

fn gen_license() -> Result<(), Box<dyn std::error::Error>> {
    run_command(
        "cargo",
        &[
            "bundle-licenses",
            "--format",
            "toml",
            "--output",
            "THIRDPARTY.toml",
        ],
        None,
    )?;
    Ok(())
}

fn run_command(
    program: &str,
    args: &[&str],
    cwd: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    run_cmd(cmd)
}

fn run_cmd(mut cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("Command failed with status: {}", status).into());
    }
    Ok(())
}

fn run_cmd_in_dir(mut cmd: Command, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    cmd.current_dir(dir);
    run_cmd(cmd)
}
