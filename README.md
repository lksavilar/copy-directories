# Backup LKS — CLI multiplataforma

Herramienta CLI en Rust para backup y restore de directorios entre el equipo y un disco externo. Usa `rsync` en Linux/macOS y `robocopy` en Windows.

## Uso rápido

```bash
# Backup
./target/release/copy_directories_iniciolukas backup

# Restore completo (pide confirmación)
./target/release/copy_directories_iniciolukas restore

# Restore de una carpeta específica
./target/release/copy_directories_iniciolukas restore --path "rust_projects"

# Restore sin confirmación
./target/release/copy_directories_iniciolukas restore --force
```

## Compilar

```bash
cargo build --release
cargo test
```

## Configuración

El nombre del binario determina qué perfil se carga:

```
copy_directories_iniciolukas
                 ^^^^^^^^^^^^  → inputs-backup/iniciolukas.json
```

### Archivos de configuración

| Archivo | Descripción |
|---------|-------------|
| `inputs-backup/iniciolukas.json` | Config principal (rutas de archivos, apagado) |
| `inputs-backup/backup_lukas_rutas.json` | Rutas origen→destino y punto de montaje |
| `inputs-backup/backup_lukas_excluir_carpetas.txt` | Patrones a excluir del backup |

### Rutas respaldadas (Linux)

| Origen | Destino en disco |
|--------|-----------------|
| `~/rust_projects/` | `/mnt/g/linux/rust_projects` |
| `~/rust-dioxus-docker/` | `/mnt/g/linux/rust-dioxus-docker` |
| `~/dioxus-spa/` | `/mnt/g/linux/dioxus-spa` |
| `~/python_projects/` | `/mnt/g/linux/python_projects` |

El disco de backup **BACKUP-LKS** se monta en `/mnt/g` (symlink a `/media/lukasar/BACKUP-LKS`).

## Logs

- **Linux:** `~/logs/backup/`
- **Windows:** `C:\xcopy\` (resumen) y `C:\xcopy\logs\` (detalle)

## Plataformas

| | Linux | Windows |
|---|---|---|
| Herramienta | `rsync -avz --itemize-changes` | `robocopy /E /J /MT:16` |
| Montaje backup | `/mnt/g/` | `G:\` |
| Apagado post-backup | No aplica | `y/s`=hibernar, `off`=apagar |

## Estructura

```
src/
├── main.rs        # Punto de entrada
├── cli.rs         # Parser de argumentos (clap)
├── platform.rs    # Detección de OS, normalización de rutas
├── commands.rs    # Construcción de comandos rsync/robocopy
├── backup.rs      # Flujo de backup
├── restore.rs     # Flujo de restore
├── config/
│   └── types.rs   # Structs Config e Inicio
└── tools.rs       # Validación de disco, fecha, config_loader
```

Ver [MANUAL_USUARIO.md](MANUAL_USUARIO.md) y [MANUAL_ADMIN.md](MANUAL_ADMIN.md) para documentación detallada.
