// This is free and unencumbered software released into the public domain.

//! Program configuration module.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use log::{info, warn};
use serde::Deserialize;

use crate::cli::Cli;

/// Program configuration data.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// Command to run inside the container.
    #[serde(default)]
    pub command: Vec<String>,
    /// Selected profile.
    #[serde(default)]
    pub profile: Option<PathBuf>,
    /// Directory-specific container profiles.
    #[serde(default)]
    pub profiles: HashMap<PathBuf, Profile>,
}

/// Container profile data.
#[derive(Debug, Deserialize)]
pub struct Profile {
    /// Run the container with a custom entrypoint.
    #[serde(default = "Profile::default_entrypoint")]
    pub entrypoint: bool,
    /// Environment variables to define in the container.
    #[serde(default)]
    pub env: HashMap<String, Option<String>>,
    /// OCI container image to use.
    pub image: String,
    /// Directories to append to the end of `PATH` (when using custom entrypoint).
    #[serde(default)]
    #[serde(rename = "path-append")]
    pub path_append: Vec<String>,
    /// Directories to prepend to the beginning of `PATH` (when using custom entrypoint).
    #[serde(default)]
    #[serde(rename = "path-prepend")]
    pub path_prepend: Vec<String>,
    /// Use setpriv to drop privileges (when using custom entrypoint).
    #[serde(default = "Profile::default_setpriv")]
    pub setpriv: bool,
    /// Unprivileged user name (when using custom entrypoint).
    #[serde(default)]
    pub user: Option<String>,
    /// Unprivileged user GID (when using custom entrypoint).
    #[serde(default)]
    #[serde(rename = "user-gid")]
    pub user_gid: Option<u32>,
    /// Unprivileged user UID (when using custom entrypoint).
    #[serde(default)]
    #[serde(rename = "user-uid")]
    pub user_uid: Option<u32>,
    /// Volumes to mount.
    #[serde(default)]
    pub volumes: Vec<String>,
    /// Working directory.
    #[serde(default)]
    pub workdir: Option<String>,
}

impl Config {
    /// Get the default path to the config file.
    fn default_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("oci-run");
            path.push("config.yaml");
            path
        })
    }
}

impl TryFrom<Cli> for Config {
    type Error = Box<dyn Error>;

    /// Try to get configuration from parsed command-line options.
    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        // Load defaults from the config file
        let mut config = if let Some(path) = cli.config_file.or_else(Self::default_config_path) {
            if path.is_file() {
                info!("loading config file: {}", path.display());
                let file = File::open(&path)?;
                let value = yaml_merge_keys::merge_keys_serde(serde_yaml::from_reader(&file)?)?;
                serde_yaml::from_value(value)?
            } else {
                warn!("using default config, file not found: {}", path.display());
                warn!("create a config file to suppress this warning");
                warn!("do you need --config-file=PATH to read from a different file?");
                Self::default()
            }
        } else {
            warn!("using default config, can't locate the config file");
            warn!("do you need to define the HOME environment variable?");
            warn!("use --config-file=PATH to manually specify the path to the config file");
            Self::default()
        };

        config.command = cli.command;
        config.profile = cli.profile;

        // Expand and canonicalize profile paths
        let mut profiles = HashMap::new();
        for (path, profile) in config.profiles {
            // Expand variables in envar values
            let mut env = HashMap::new();
            for (name, value) in profile.env {
                let value = if let Some(value) = value {
                    Some(shellexpand::full(&value)?.to_string())
                } else {
                    None
                };
                env.insert(name, value);
            }
            let profile = Profile { env, ..profile };

            // Expand variables in the profile path
            let path = path
                .to_str()
                .ok_or_else(|| format!("profile path must be utf-8: {}", path.display()))?;
            let path = shellexpand::full(path)?.to_string();
            let path = PathBuf::from(path);

            // Try to canonicalize the path before adding the profile
            if let Ok(path) = path.canonicalize() {
                profiles.insert(path, profile);
            } else {
                info!("non-canonical profile: {}", path.display());
                profiles.insert(path, profile);
            }
        }
        config.profiles = profiles;

        Ok(config)
    }
}

impl Profile {
    /// Get the default entrypoint value.
    fn default_entrypoint() -> bool {
        true
    }

    /// Get the default setpriv value.
    fn default_setpriv() -> bool {
        true
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            entrypoint: Self::default_entrypoint(),
            env: HashMap::default(),
            image: String::default(),
            path_append: Vec::default(),
            path_prepend: Vec::default(),
            setpriv: Self::default_setpriv(),
            user: Option::default(),
            user_gid: Option::default(),
            user_uid: Option::default(),
            volumes: Vec::default(),
            workdir: Option::default(),
        }
    }
}
