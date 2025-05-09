use anyhow::Result;

/* -------------------------------------------- CLI -------------------------------------------- */

// TODO

/* -------------------------------------------- MAIN ------------------------------------------- */

/// The main entrypoint for the pixi-install-to-prefix CLI.
#[tokio::main]
async fn main() -> Result<()> {
    // let cli = Cli::parse();

    // tracing_subscriber::FmtSubscriber::builder()
    //     .with_max_level(cli.verbose)
    //     .init();

    tracing::debug!("Starting pixi-install-to-prefix CLI");

    todo!();

    tracing::debug!("Finished running pixi-install-to-prefix");

    Ok(())
}
