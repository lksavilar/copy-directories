# CLAUDE.md — Backup LKS

## Propósito

Herramienta CLI en Rust para backup y restore de directorios entre el equipo y un disco externo **BACKUP-LKS**. Multiplataforma: usa `rsync` en Linux/macOS y `robocopy` en Windows.

## Comandos esenciales

```bash
cargo build --release          # Compilar producción
cargo run -- backup            # Ejecutar backup
cargo run -- restore           # Restaurar (con confirmación)
cargo run -- restore --path "nombre"   # Restaurar carpeta específica
cargo run -- restore --force   # Restaurar sin confirmación
cargo test                     # Correr 57 tests unitarios
cargo test -- --ignored        # + test con disco real
```

## Arquitectura

```
main.rs → cli.rs (clap)
       → backup.rs  ─┐
       → restore.rs ─┤→ platform.rs (OS detection)
                     ├→ commands.rs (rsync / robocopy)
                     ├→ tools.rs   (validación disco, fecha, config_loader)
                     └→ config/types.rs (Config, Inicio structs)
```

## Naming del binario

```
copy_directories_iniciolukas
                 ^^^^^^^^^^^^  → lee inputs-backup/iniciolukas.json
```

Cambiar el tercer segmento para cargar un perfil distinto.

## Configuración activa (Linux)

- **Inicio:** `inputs-backup/iniciolukas.json`
- **Rutas:** `inputs-backup/backup_lukas_rutas.json`
- **Exclusiones:** `inputs-backup/backup_lukas_excluir_carpetas.txt`
- **Disco backup:** `BACKUP-LKS` montado en `/mnt/g` (symlink → `/media/lukasar/BACKUP-LKS`)
- **Destinos:** `/mnt/g/linux/{rust_projects,rust-dioxus-docker,dioxus-spa,python_projects}`

## Logs

- Linux: `~/logs/backup/`
- Windows: `C:\xcopy\` y `C:\xcopy\logs\`

## Validación de disco al arrancar

`tools::check_drive_backup` verifica en orden:
1. Directorio existe
2. Hay filesystem montado (`esta_montado()` — compara `dev()` con padre)
3. Permisos de escritura

## Plataforma

| | Linux | Windows |
|---|---|---|
| Herramienta | `rsync -avz --itemize-changes` | `robocopy /E /J /MT:16` |
| Logs | `~/logs/backup/` | `C:\xcopy\` |
| Montaje backup | `/mnt/g/` (symlink) | `G:\` |
| Apagado post-backup | No aplica | `y/s`=hibernar, `off`=apagar |

## Tests

57 tests unitarios en `platform`, `commands`, `config::types`, `tools::check_drive_backup`.  
Sin mocks — pruebas sobre lógica pura y filesystem real (temp dirs).

## Archivos de exclusión

`backup_lukas_excluir_carpetas.txt` soporta comentarios (`#`) y líneas vacías.  
Se filtran antes de pasar a rsync/robocopy.

## No modificar sin entender

- `extraer_letra_drive()` — parsing de letra de drive Windows desde `destino_final`
- `esta_montado()` — usa `MetadataExt::dev()`, solo Linux/macOS
- Códigos de éxito robocopy: **0-7 = éxito**, 8+ = error
