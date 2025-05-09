use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use pixi_config::Config;
use pixi_install_to_prefix::reqwest_client_from_config;
use rattler::install::Installer;
use rattler_conda_types::{Platform, RepoDataRecord};
use rattler_lock::{
    CondaPackageData, DEFAULT_ENVIRONMENT_NAME, LockFile, LockedPackageRef, UrlOrPath,
};
use tokio::fs::read_to_string;

/* -------------------------------------------- CLI -------------------------------------------- */

#[derive(Parser, Debug)]
struct Cli {
    /// The path to the prefix where you want to install the environment.
    #[clap()]
    prefix: String,

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

    #[command(flatten)]
    verbose: Verbosity,
}

/* -------------------------------------------- MAIN ------------------------------------------- */

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
        let config_str = read_to_string(&config_path).await?;
        let (config, _unused_keys) =
            Config::from_toml(config_str.as_str(), Some(&config_path.clone()))
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
                        UrlOrPath::Path(_) => Err(anyhow!("Path package {} is not supported", p.location))?,
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
        .with_reporter(
            rattler::install::IndicatifReporter::builder()
                // .with_multi_progress(global_multi_progress())
                .finish(),
        )
        .install(&cli.prefix, packages)
        .await?;

    eprintln!(
        "Installed {} packages to {}",
        result.transaction.operations.len(),
        cli.prefix
    );

    tracing::debug!("Finished running pixi-install-to-prefix");

    Ok(())
}
