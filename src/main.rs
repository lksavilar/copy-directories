use std::io;

// Declarar módulos
mod cli;
mod config;
mod tools;
mod backup;
mod restore;
mod platform;
mod commands;

use cli::{parse_args, Commands};

fn main() -> io::Result<()> {
    // Si no hay argumentos, ejecutar backup por defecto (comportamiento original)
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() == 1 {
        println!("🔄 Iniciando proceso de backup (modo compatibilidad)...");
        return backup::run_backup();
    }
    
    let cli = parse_args();

    match cli.command {
        Commands::Backup => {
            println!("🔄 Iniciando proceso de backup...");
            backup::run_backup()
        }
        Commands::Restore { path, force } => {
            println!("🔄 Iniciando proceso de restore...");
            restore::run_restore(path, force)
        }
    }
}
