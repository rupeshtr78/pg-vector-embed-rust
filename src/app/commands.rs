use super::constants::{EMBEDDING_MODEL, VECTOR_DB_DIM_STR, VECTOR_DB_TABLE, VERSION};
use clap::{Parser, Subcommand, ValueEnum};
use log::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[clap(name = "pg-vector-embed-rust")]
#[clap(about = "A CLI application for embedding and querying a vector database", long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Option<Commands>,
    #[clap(short, long, global = true)]
    log_level: Option<LogLevel>,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Write embedding to Postgres Vector Database
    Write {
        /// provide the input string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use
        #[clap(default_value = &EMBEDDING_MODEL)]
        #[clap(short, long)]
        model: String,
        /// Provide the table to use default is "from_rust"
        #[clap(default_value = VECTOR_DB_TABLE)]
        #[clap(short, long)]
        table: String,
        /// Provide the vector dimension to use default is 768
        #[clap(default_value = VECTOR_DB_DIM_STR)]
        #[clap(short, long)]
        dim: String,
    },

    /// Query the Vector Database   
    Query {
        /// The query string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use for query embedding
        #[clap(short, long)]
        #[clap(default_value = EMBEDDING_MODEL)]
        model: String,
        /// Provide the table to use to query
        #[clap(short, long)]
        #[clap(default_value = VECTOR_DB_TABLE)]
        table: String,
    },
    /// Get the version of the application
    Version {
        /// The version of the application
        #[clap(short, long)]
        #[clap(default_value = VERSION)]
        version: String,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Commands {
    /// Checks if the command is a `Write` command.
    pub fn is_write(&self) -> bool {
        matches!(self, Commands::Write { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Write` command.
    pub fn write(&self) -> Option<&Commands> {
        if let Commands::Write { .. } = self {
            Some(self)
        } else {
            None
        }
    }

    /// Checks if the command is a `Query` command.
    pub fn is_query(&self) -> bool {
        matches!(self, Commands::Query { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Query` command.
    pub fn query(&self) -> Option<&Commands> {
        if let Commands::Query { .. } = self {
            Some(self)
        } else {
            None
        }
    }

    /// Checks if the command is a `Version` command.
    pub fn is_version(&self) -> bool {
        matches!(self, Commands::Version { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Version` command.
    pub fn version(&self) -> Option<&Commands> {
        if let Commands::Version { .. } = self {
            Some(self)
        } else {
            None
        }
    }
}

impl LogLevel {
    // map loglevel to log::LevelFilter
    pub fn get_log_level_filter(&self) -> log::LevelFilter {
        match self {
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
        }
    }
}

fn colog_init(log_level: LogLevel) {
    // colog::basic_builder()
    //     .filter_level(log_level.get_log_level_filter())
    //     .format_module_path(true)
    //     .init();

    // Initialize env_logger with module path formatting
    let mut builder = env_logger::Builder::new();
    builder
        .filter_level(log_level.get_log_level_filter())
        .format_module_path(true)
        .init();

    println!("Log level set to: {:?}", log_level);
}

pub fn build_args() -> Commands {
    let args = Args::parse();

    // Handle log level (if provided)
    if let Some(log_level) = args.log_level {
        match log_level {
            LogLevel::Error => colog_init(LogLevel::Error),
            LogLevel::Warn => colog_init(LogLevel::Warn),
            LogLevel::Info => colog_init(LogLevel::Info),
            LogLevel::Debug => colog_init(LogLevel::Debug),
        }
    } else {
        colog_init(LogLevel::Info);
    }

    let commands = match args.cmd {
        Some(command) => command,
        None => {
            info!("No subcommand provided. Use --help for more information.");
            return Commands::Empty;
        }
    };

    commands
}

/// quick and dirty way to test the command line arguments
#[allow(dead_code)]
pub fn dbg_cmd() {
    // cargo run -- help
    // cargo run -- write --help
    // cargo run -- write --input "hello" "world"
    // cargo run -- write --input "hello","world" --model "nomic-embed-text1" --table "from_rust2" --dim 7681
    // cargo run -- write --input "dog sound is called bark" --input "cat sounds is called purr" --model "nomic-embed-text"
    // cargo run -- write --input "dog sound is called bark" --input "cat sounds is called purr" --model "nomic-embed-text" --table "from_rust2" --dim 768
    // cargo run -- query --input "who is barking" --model "nomic-embed-text" --table "from_rust2"
    // cargo test --package pg-vector-embed-rust --lib -- tests::pgclient_test::pg_client_tests --show-output
    let args = Args::parse();
    println!("Parsed args: {:?}", args);
    let commands = match args.cmd {
        Some(command) => command,
        None => {
            println!("No subcommand provided. Use --help for more information.");
            return;
        }
    };

    match &commands {
        Commands::Write {
            input,
            model: embed_model,
            table,
            dim,
        } => {
            println!("Write command");
            println!("Input: {:?}", input);
            println!("Model: {:?}", embed_model);
            println!("Table: {:?}", table);
            println!("Dimension: {:?}", dim);
        }
        Commands::Query {
            input,
            model,
            table,
        } => {
            println!("Query command");
            println!("Query: {:?}", input);
            println!("Model: {:?}", model);
            println!("Table: {:?}", table);
        }
        Commands::Version { version } => {
            println!("Version command");
            println!("Version: {:?}", version);
        }
        Commands::Empty => {
            println!("Empty command");
        }
    }
}
