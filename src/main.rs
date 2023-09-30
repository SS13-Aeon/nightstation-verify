use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use nightstation_verify::{App, AppConfig, AppConfigError, AppError, AppPaths};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Config file
    #[arg(short, long, default_value = "config.ron")]
    config: PathBuf,
    /// Data directory
    #[arg(short, long, default_value = "data")]
    data: PathBuf,
    /// Don't run after loading or creating config
    #[arg(short, long)]
    no_run: bool,
    /// Print copyright notice
    #[arg(short = 'C', long)]
    copyright: bool,
    /// Print source code link
    #[arg(short = 'S', long)]
    source: bool,
    /// Increase verbosity level
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.copyright {
        eprintln!("{}", nightstation_verify::COPYRIGHT);

        return ExitCode::SUCCESS;
    }

    if cli.source {
        eprintln!(
            "Copies of the source code can be obtained from {}",
            nightstation_verify::SOURCE
        );

        return ExitCode::SUCCESS;
    }

    match cli.verbose {
        0 => {
            stderrlog::new()
                .module(module_path!())
                .verbosity(log::Level::Info)
                .init()
                .unwrap();
        }
        1 => {
            stderrlog::new()
                .show_module_names(true)
                .verbosity(log::Level::Info)
                .init()
                .unwrap();
        }
        2 => {
            stderrlog::new()
                .show_module_names(true)
                .verbosity(log::Level::Debug)
                .init()
                .unwrap();
        }
        _ => {
            stderrlog::new()
                .show_module_names(true)
                .verbosity(log::Level::Trace)
                .init()
                .unwrap();
        }
    };

    let config = match AppConfig::load(Path::new(&cli.config)).await {
        Ok(config) => {
            log::info!(
                "Loaded config from {}",
                fs::canonicalize(&cli.config)
                    .expect("Error while canonicalizing config path")
                    .display()
            );

            config
        }
        Err(e) => {
            match e {
                AppConfigError::Io(e) => log::error!("Error while opening config file: {}", e),
                AppConfigError::Parse(e) => log::error!("Error while parsing config file: {}", e),
                AppConfigError::Serialize(e) => {
                    log::error!("Error while writing config file: {}", e)
                }
                AppConfigError::DiscordError(e) => log::error!("Unexpected Discord error: {}", e),
                AppConfigError::WizardDismissed => {
                    log::error!("Cannot continue without configuration")
                }
            };

            return ExitCode::FAILURE;
        }
    };

    if cli.no_run {
        return ExitCode::SUCCESS;
    }

    let app = App::new(
        config,
        AppPaths {
            config: Path::new(&cli.config),
            data: Path::new(&cli.data),
        },
    );

    match app.run().await {
        Ok(_) => (),
        Err(e) => {
            match e {
                AppError::DataDir(e) => log::error!("Error while creating data directory: {}", e),
                AppError::Whietlist(e) => log::error!("Error while reading whitelist: {}", e),
                AppError::Discord(e) => log::error!("Unexpected Discord error: {}", e),
                AppError::Service(e) => log::error!("Service error: {}", e),
                AppError::JoinError(e) => log::error!("Tokio error: {}", e),
            };

            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
