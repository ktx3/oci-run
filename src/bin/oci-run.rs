// This is free and unencumbered software released into the public domain.

//! Tool for running OCI containers.

use std::convert::TryFrom;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::process::{self, Command};

use clap::Parser;
use crossterm::tty::IsTty;
use fern::Dispatch;
use log::{debug, LevelFilter};
use oci_run::{Cli, Config};

/// Get program configuration from command-line arguments.
fn get_config() -> Result<Config, Box<dyn Error>> {
    let cli = Cli::parse();

    // Configure logging before handling other options
    let level = match cli.verbosity() {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Dispatch::new()
        .format(|out, msg, rec| {
            out.finish(format_args!("oci-run: [{}] {}", rec.level(), msg));
        })
        .level(if cli.debug() { level } else { LevelFilter::Off })
        .level_for("oci_run", level)
        .chain(io::stderr())
        .apply()?;

    let config = Config::try_from(cli)?;
    debug!("config: {:#?}", config);
    Ok(config)
}

/// Run command in a container using configuration profile for the current directory.
fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?.canonicalize()?;
    let profile_path = config.profile.as_ref().unwrap_or(&current_dir);
    let profile = config
        .profiles
        .get(profile_path)
        .ok_or_else(|| format!("no profile: {}", profile_path.display()))?;

    // Start building the command to be executed
    let mut cmd = [
        "docker",
        "container",
        "run",
        "--init",
        "--interactive",
        "--rm",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<_>>();

    // Detect if stdin is coming from a terminal or not
    if io::stdin().is_tty() {
        cmd.push("--tty".to_string());
    }

    // Privilege dropping
    cmd.push(format!(
        "--env=DISABLE_SETPRIV={}",
        if profile.setpriv { "" } else { "1" }
    ));

    if let Some(user) = &profile.user {
        cmd.push(format!("--env=USER_NAME={}", user));
    }

    cmd.push(format!(
        "--env=USER_GID={}",
        profile.user_gid.unwrap_or_else(users::get_effective_gid)
    ));

    cmd.push(format!(
        "--env=USER_UID={}",
        profile.user_uid.unwrap_or_else(users::get_effective_uid)
    ));

    // PATH configuration
    if !profile.path_append.is_empty() {
        cmd.push(format!(
            "--env=USER_PATH_POST={}",
            profile.path_append.join(":")
        ));
    }

    if !profile.path_prepend.is_empty() {
        cmd.push(format!(
            "--env=USER_PATH_PRE={}",
            profile.path_prepend.join(":")
        ));
    }

    // Environment variables
    for (key, value) in &profile.env {
        if let Some(s) = value {
            cmd.push(format!("--env={}={}", key, s));
        } else {
            cmd.push(format!("--env={}", key));
        }
    }

    // Workdir
    if let Some(workdir) = &profile.workdir {
        cmd.push(format!("--workdir={}", workdir));
    }

    // Volume mounts
    for volume in &profile.volumes {
        cmd.push(format!("--volume={}", shellexpand::full(volume)?));
    }

    // Custom entrypoint
    if profile.entrypoint {
        let entrypoint = {
            // Path to the entrypoint script that will be mounted in the container
            let path = dirs::cache_dir()
                .map(|mut path| {
                    path.push("oci-run");
                    path.push("oci-entrypoint");
                    path
                })
                .ok_or(
                    "can't locate cache dir (do you need to define the HOME environment variable?)",
                )?;

            // Create the entrypoint script
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut file = File::create(&path)?;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            file.set_permissions(perms)?;
            file.write_all(include_bytes!("../../oci-entrypoint/oci-entrypoint.sh"))?;
            file.flush()?;

            path
        };

        cmd.push(format!(
            "--volume={}:/usr/local/bin/oci-entrypoint:ro",
            entrypoint.display()
        ));
        cmd.push("--entrypoint=/usr/local/bin/oci-entrypoint".to_string());
    }

    // Image and inner command arguments
    cmd.push("--".to_string());
    cmd.push(profile.image.clone());
    cmd.extend(config.command.iter().cloned());

    debug!("command: {:#?}", cmd);
    Command::new(&cmd[0]).args(&cmd[1..]).exec();
    Err(format!("command failed: {:?}", cmd).into())
}

/// Main entrypoint.
fn main() {
    if let Err(err) = get_config().and_then(|config| run(&config)) {
        eprintln!("oci-run: {}", err);
        process::exit(1);
    }
}
