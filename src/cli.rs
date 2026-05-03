use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "backup_directories")]
#[command(about = "Una aplicación para hacer backup y restore de directorios")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Realizar backup de los directorios configurados
    Backup,
    /// Restaurar directorios desde el backup
    Restore {
        /// Ruta específica a restaurar (opcional)
        #[arg(short, long)]
        path: Option<String>,
        /// Restaurar todo sin confirmación
        #[arg(short, long)]
        force: bool,
    },
}

pub fn parse_args() -> Cli {
    Cli::parse()
}