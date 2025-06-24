use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use pixi_install_to_prefix::{Config, create_activation_scripts, reqwest_client_from_config};
use rattler::install::Installer;
use rattler_conda_types::{Platform, RepoDataRecord};
use rattler_lock::{
    CondaPackageData, DEFAULT_ENVIRONMENT_NAME, LockFile, LockedPackageRef, UrlOrPath,
};
use rattler_shell::shell::{Bash, CmdExe, Fish, PowerShell, ShellEnum};
use tokio::fs::{self, OpenOptions};

/* -------------------------------------------- CLI -------------------------------------------- */

#[derive(Parser, Debug)]
struct Cli {
    /// The path to the prefix where you want to install the environment.
    #[clap()]
    prefix: PathBuf,

    /// The path to the pixi lockfile.
    #[arg(short, long, default_value = "pixi.lock")]
    lockfile: PathBuf,

    /// The name of the pixi environment to install.
    #[arg(short, long, default_value = DEFAULT_ENVIRONMENT_NAME)]
    environment: String,

    /// The platform you want to install for.
    #[arg(short, long, default_value = Platform::current().to_string())]
    platform: Platform,

    /// The path to the pixi config file. By default, no config file is used.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// The shell(s) to generate activation scripts for. Default: see README.
    #[arg(short, long)]
    shell: Option<Vec<ShellEnum>>,

    /// Disable the generation of activation scripts.
    #[arg(long, conflicts_with = "shell")]
    no_activation_scripts: bool,

    #[command(flatten)]
    verbose: Verbosity,
}

/* -------------------------------------------- MAIN ------------------------------------------- */

const CONDA_HISTORY_FILE: &str = "conda-meta/history";

/// The main entrypoint for the pixi-install-to-prefix CLI.
#[tokio::main]
async fn main() -> Result<()> {
    tracing::debug!("Starting pixi-install-to-prefix CLI");
    let cli = Cli::parse();

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(cli.verbose)
        .init();

    let pixi_config = if let Some(config_path) = cli.config {
        tracing::debug!("Using config file: {}", config_path.display());
        let config = Config::load_from_files(vec![&config_path.clone()])
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;
        Some(config)
    } else {
        None
    };

    let download_client = reqwest_client_from_config(&pixi_config)
        .map_err(|e| anyhow::anyhow!("Failed to create download client: {}", e))?;

    // TODO: throw error on virtual package mismatch?

    let lockfile = LockFile::from_path(&cli.lockfile).map_err(|e| {
        anyhow!(
            "could not read lockfile at {}: {}",
            cli.lockfile.display(),
            e
        )
    })?;

    let environment = lockfile
        .environment(cli.environment.as_str())
        .ok_or(anyhow!(
            "Environment {} not found in lockfile",
            cli.environment
        ))?;
    let packages = environment.packages(cli.platform).ok_or(anyhow!(
        "environment {} does not contain platform {}",
        cli.environment,
        cli.platform
    ))?;

    let packages = packages
        .map(|p| match p {
            LockedPackageRef::Conda(p) => match p {
                CondaPackageData::Binary(p) => Ok(RepoDataRecord {
                    package_record: p.package_record.clone(),
                    file_name: p.file_name.clone(),
                    url: match p.location.clone() {
                        UrlOrPath::Url(url) => url,
                        UrlOrPath::Path(_) => {
                            Err(anyhow!("Path package {} is not supported", p.location))?
                        }
                    },
                    channel: p.channel.clone().map(|c| c.to_string()),
                }),
                CondaPackageData::Source(p) => {
                    Err(anyhow!("Source package {} is not supported", p.location))
                }
            },
            LockedPackageRef::Pypi(_, _) => {
                Err(anyhow!("Pypi package {} is not supported", p.location()))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let result = Installer::new()
        .with_download_client(download_client)
        .with_target_platform(cli.platform)
        .with_execute_link_scripts(true)
        .with_reporter(rattler::install::IndicatifReporter::builder().finish())
        .install(&cli.prefix, packages)
        .await?;

    // hotfix: create history file, otherwise the prefix is rejected by conda
    // TODO: can be removed with rattler-conda-types v0.33.0
    OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(cli.prefix.join(CONDA_HISTORY_FILE))
        .await?;

    eprintln!(
        "Installed {} packages to {}",
        result.transaction.operations.len(),
        cli.prefix.display()
    );

    if !cli.no_activation_scripts {
        let shells = cli.shell.unwrap_or_else(|| {
            // Default shells based on the platform
            match cli.platform {
                Platform::Win64 | Platform::Win32 | Platform::WinArm64 => {
                    vec![CmdExe.into(), PowerShell::default().into(), Bash.into()]
                }
                _ => vec![Bash.into(), Fish.into()],
            }
        });
        create_activation_scripts(&fs::canonicalize(&cli.prefix).await?, shells, cli.platform)
            .await?;
    } else {
        tracing::debug!("Skipping activation script generation as requested");
    }

    tracing::debug!("Finished running pixi-install-to-prefix");

    Ok(())
}
