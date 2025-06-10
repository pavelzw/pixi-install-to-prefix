use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{Result, anyhow};

use rattler_conda_types::Platform;
use rattler_networking::{
    AuthenticationMiddleware, AuthenticationStorage, MirrorMiddleware, S3Middleware,
    mirror_middleware::Mirror,
};
use rattler_shell::{
    activation::{ActivationVariables, Activator, PathModificationBehavior},
    shell::{Shell, ShellEnum},
};
use reqwest_middleware::ClientWithMiddleware;
use tokio::fs;

/// Create a reqwest client (optionally including authentication middleware).
pub fn reqwest_client_from_config(
    config: &Option<pixi_config::Config>,
) -> Result<ClientWithMiddleware> {
    let auth_storage = AuthenticationStorage::from_env_and_defaults()?;

    let s3_middleware = if let Some(config) = config {
        let s3_config = config.compute_s3_config();
        tracing::info!("Using S3 config: {:?}", s3_config);
        S3Middleware::new(s3_config, auth_storage.clone())
    } else {
        S3Middleware::new(HashMap::new(), auth_storage.clone())
    };
    let mirror_middleware = if let Some(config) = config {
        let mut internal_map = HashMap::new();
        tracing::info!("Using mirrors: {:?}", config.mirror_map());

        fn ensure_trailing_slash(url: &url::Url) -> url::Url {
            if url.path().ends_with('/') {
                url.clone()
            } else {
                // Do not use `join` because it removes the last element
                format!("{}/", url)
                    .parse()
                    .expect("Failed to add trailing slash to URL")
            }
        }
        for (key, value) in config.mirror_map() {
            let mut mirrors = Vec::new();
            for v in value {
                mirrors.push(Mirror {
                    url: ensure_trailing_slash(v),
                    no_jlap: false,
                    no_bz2: false,
                    no_zstd: false,
                    max_failures: None,
                });
            }
            internal_map.insert(ensure_trailing_slash(key), mirrors);
        }
        MirrorMiddleware::from_map(internal_map)
    } else {
        MirrorMiddleware::from_map(HashMap::new())
    };

    let timeout = 5 * 60;
    let client = reqwest_middleware::ClientBuilder::new(
        reqwest::Client::builder()
            .no_gzip()
            .pool_max_idle_per_host(20)
            .user_agent(format!(
                "pixi-install-to-prefix/{}",
                env!("CARGO_PKG_VERSION")
            ))
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .map_err(|e| anyhow!("could not create download client: {}", e))?,
    )
    .with(mirror_middleware)
    .with(s3_middleware)
    .with_arc(Arc::new(AuthenticationMiddleware::from_auth_storage(
        auth_storage,
    )))
    .build();
    Ok(client)
}

pub async fn create_activation_scripts(
    prefix: &Path,
    shells: Vec<ShellEnum>,
    platform: Platform,
) -> Result<()> {
    for shell in shells {
        let file_extension = shell.extension();
        let parent_dir = prefix.join("conda-meta/activation");
        // Ensure the parent directory exists
        fs::create_dir_all(&parent_dir)
            .await
            .map_err(|e| anyhow!("Could not create activation directory: {}", e))?;
        let destination = parent_dir.join(format!("activate.{}", file_extension));
        let activator = Activator::from_path(prefix, shell.clone(), platform)?;
        let result = activator.activation(ActivationVariables {
            conda_prefix: None,
            path: None,
            path_modification_behavior: PathModificationBehavior::Prepend,
        })?;
        let contents = result.script.contents()?;

        fs::write(&destination, contents)
            .await
            .map_err(|e| anyhow!("Could not write activate script: {}", e))?;

        tracing::info!(
            "Activation script for {:?} created at {}",
            shell,
            destination.display()
        );
    }

    Ok(())
}
