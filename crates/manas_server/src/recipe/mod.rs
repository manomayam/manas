//! I provide utilities to construct and serve integrated
//! recipes.
//!

use std::{borrow::Cow, path::PathBuf};

use clap::{ArgAction, Command};
use config::{Config, FileFormat};
use futures::{future::BoxFuture, TryFutureExt};
use tower::BoxError;
use tracing::error;

use crate::tracing::{get_subscriber, init_subscriber};

pub mod impl_;

/// A trait for representing integrated recipe.
pub trait Recipe: Send + Sync + 'static {
    /// Type of the config.
    type Config: Send + Sync + 'static + serde::Serialize + for<'a> serde::Deserialize<'a>;

    /// Cli name of the recipe.
    fn cli_name(&self) -> Cow<'static, str>;

    /// Description of the recipe.
    fn description(&self) -> Cow<'static, str>;

    /// Serve the recipe with parsed config.
    fn serve(&self, config: Self::Config) -> BoxFuture<'static, Result<(), BoxError>>;
}

/// An extension trait for [`Recipe`].
pub trait RecipeExt: Recipe {
    /// Get a simple cli command that reads config file path.
    fn cli_command(&self) -> Command {
        Command::new(self.cli_name().into_owned())
            .about(self.description().into_owned())
            .arg(
                clap::arg!(
                    -c --config <FILE> "Sets a custom config file"
                )
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                clap::arg!(
                    -d --debug ... "Turn debugging information on"
                )
                .action(ArgAction::SetTrue),
            )
    }

    /// Parse cli args.
    fn parse_cli_args(&self) -> Result<RecipeCliArgs, BoxError> {
        let mut cli = self.cli_command().get_matches();
        Ok(RecipeCliArgs {
            config_path: cli
                .remove_one::<PathBuf>("config")
                .ok_or("Config is required.")?,
            log_level: if cli.get_flag("debug") {
                "debug"
            } else {
                "info"
            }
            .to_owned(),
        })
    }

    /// Run the recipe.
    fn run(&self, args: RecipeCliArgs) -> BoxFuture<'_, Result<(), BoxError>> {
        Box::pin(async move {
            // Enable tracing.
            init_subscriber(get_subscriber("Manas".to_owned(), args.log_level));

            // Resolve config.
            let config_content = String::from_utf8(
                tokio::fs::read(args.config_path)
                    .inspect_err(|e| {
                        error!("Error in reading config fle. {}", e);
                    })
                    .await?,
            )
            .map_err(|_| "Invalid config file content")?;

            let config = Config::builder()
                .add_source(config::File::from_str(&config_content, FileFormat::Toml))
                .build()?
                .try_deserialize::<Self::Config>()
                .map_err(|e| {
                    error!("Error in parsing configuration. Error: {}", e);
                    e
                })?;

            // Serve.
            self.serve(config).await
        })
    }

    /// Run the recipe.
    #[inline]
    fn main(&self) -> BoxFuture<'_, Result<(), BoxError>> {
        Box::pin(async move { self.run(self.parse_cli_args()?).await })
    }
}

impl<R: Recipe> RecipeExt for R {}

/// Cli args for recipe.
#[derive(Debug, Clone)]
pub struct RecipeCliArgs {
    /// Recipe configuration path.
    pub config_path: PathBuf,

    /// Log level.
    pub log_level: String,
}
