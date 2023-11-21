use std::path::PathBuf;

use clap::{ArgAction, Args, Parser};
use lazy_static::lazy_static;
use resolve_path::PathResolveExt;
use which::which;

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

#[derive(Debug, Clone)]
pub struct Config {
    pub run: RunConfig,
    pub app: AppConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub dependencies: DependenciesConfig,
}

impl Config {
    fn new() -> Self {
        let args = Cli::parse();

        let mut config = Self {
            run: args.run,
            app: args.app,
            server: args.server,
            database: args.database,
            dependencies: args.dependencies,
        };

        config.merge_defaults();
        config.check();

        config
    }

    fn check(&self) -> &Self {
        // self.directory
        {
            assert!(
                self.app.directory.exists(),
                "The directory {:?} does not exist",
                self.app.directory
            );

            assert!(
                self.app.directory.is_dir(),
                "The directory {:?} is not a directory",
                self.app.directory
            );
        }

        // self.directory_metadata
        {
            if !self.app.metadata_directory.exists() {
                std::fs::create_dir_all(&self.app.metadata_directory).unwrap();
            }

            assert!(
                self.app.metadata_directory.exists(),
                "The metadata directory {:?} does not exist",
                self.app.metadata_directory
            );

            assert!(
                self.app.metadata_directory.is_dir(),
                "The metadata directory {:?} is not a directory",
                self.app.metadata_directory
            );
        }

        // self.secret_key
        {
            assert!(
                self.server.secret_key.len() >= 64,
                "secret_key must be at least 64 bytes long"
            );
        }

        self
    }

    fn merge_defaults(&mut self) -> &Self {
        self.app.directory = self.app.directory.try_resolve().unwrap().into();

        if self.app.metadata_directory.as_os_str().is_empty() {
            self.app.metadata_directory = self.app.directory.join(".mw_metadata");
        }
        self.app.metadata_directory = self.app.metadata_directory.try_resolve().unwrap().into();

        if self.dependencies.ffprobe_path.is_none() {
            let cmd = "ffprobe";

            self.dependencies.ffprobe_path = which(cmd)
                .ok()
                .or_else(|| which(format!("./{cmd}")).ok())
                .or_else(|| which(format!("./{cmd}.exe")).ok());
        }
        if self.dependencies.ffmpeg_path.is_none() {
            let cmd = "ffmpeg";

            self.dependencies.ffmpeg_path = which(cmd)
                .ok()
                .or_else(|| which(format!("./{cmd}")).ok())
                .or_else(|| which(format!("./{cmd}.exe")).ok());
        }

        self
    }

    #[must_use]
    pub fn db_path(&self) -> PathBuf {
        self.app.metadata_directory.join("db.sqlite3")
    }
}

#[derive(Debug, Clone, Args)]
#[clap(next_help_heading = "Run options")]
pub struct RunConfig {
    /// Generate schema types for app
    #[clap(
        short,
        long,
        default_value = "false",
        env = "MEME_WATCHER_GENERATE_TYPES"
    )]
    pub generate_types: bool,
}

#[derive(Debug, Clone, Args)]
#[clap(next_help_heading = "App options")]
pub struct AppConfig {
    /// The directory which to watch for the archive part.
    ///
    /// aka. the directory where the memes/other files are stored.
    #[arg(short, long, env = "MEME_WATCHER_DIRECTORY")]
    pub directory: PathBuf,

    /// A directory in which to store the metadata for the memes.
    ///
    /// Defaults to a `$DIRECTORY/.mw_metadata`
    #[arg(
        long,
        env = "MEME_WATCHER_METADATA_DIRECTORY",
        default_value = "$MEME_WATCHER__DIRECTORY/.mw_metadata"
    )]
    pub metadata_directory: PathBuf,
}

#[derive(Debug, Clone, Args)]
#[clap(next_help_heading = "Server options")]
pub struct ServerConfig {
    /// Host to listen on
    #[clap(short, long, default_value = "127.0.0.1", env = "HOST")]
    pub host: String,

    /// Port to listen on
    #[clap(short, long, default_value = "3001", env = "PORT")]
    pub port: u16,

    /// Secret key for signing cookies and other sensitive data
    #[clap(long, env = "SECRET_KEY")]
    pub secret_key: String,
}

#[derive(Debug, Clone, Args)]
#[clap(next_help_heading = "Database options")]
pub struct DatabaseConfig {
    /// Database URL.
    ///
    /// Should be in the format of eg. `sqlite:///absolute/path/to/database.sqlite` or just `sqlite://./path/to/database.sqlite`.
    ///
    /// If not specified, an in-memory database will be used.
    #[clap(long = "database-url", env = "DATABASE_URL")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Args)]
#[clap(next_help_heading = "Dependency options")]
pub struct DependenciesConfig {
    /// Path to ffmpeg executable
    ///
    /// If left empty, ffmpeg will be searched for in `$PATH`
    /// or in the current working directory.
    #[clap(long, env = "MW_FFMPEG_PATH")]
    pub ffmpeg_path: Option<PathBuf>,

    /// Path to ffprobe executable
    ///
    /// If left empty, ffprobe will be searched for in `$PATH`
    /// or in the current working directory.
    #[clap(long, env = "MW_FFPROBE_PATH")]
    pub ffprobe_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Parser)]
#[clap(disable_help_flag = true)]
struct Cli {
    /// Print help
    #[clap(action = ArgAction::Help, long)]
    help: Option<bool>,

    #[command(flatten)]
    run: RunConfig,

    #[command(flatten)]
    app: AppConfig,

    #[command(flatten)]
    server: ServerConfig,

    #[command(flatten)]
    database: DatabaseConfig,

    #[command(flatten)]
    dependencies: DependenciesConfig,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;

    Cli::command().debug_assert();
}
