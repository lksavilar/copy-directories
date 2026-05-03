# Manual de Administrador — Backup LKS

## Estructura del proyecto

```
copy_directories/
├── src/
│   ├── main.rs          # Punto de entrada, detección de argumentos
│   ├── cli.rs           # Parser de argumentos (clap)
│   ├── platform.rs      # Detección de OS, separadores, normalización de rutas
│   ├── commands.rs      # Construcción de comandos rsync/robocopy
│   ├── backup.rs        # Flujo principal de backup
│   ├── restore.rs       # Flujo principal de restore
│   ├── config/
│   │   ├── mod.rs       # Re-export
│   │   └── types.rs     # Structs: Config, Inicio
│   └── tools.rs         # Validación de disco, utilidades, fecha, config_loader
├── inputs-backup/
│   ├── iniciolukas.json                  # Config principal Linux
│   ├── iniciolukas_windows.json          # Config principal Windows
│   ├── backup_lukas_rutas.json           # Rutas origen→destino Linux
│   ├── backup_lukas_rutas_windows.json   # Rutas origen→destino Windows
│   └── backup_lukas_excluir_carpetas.txt # Carpetas/archivos a excluir
├── target/release/
│   └── copy_directories_iniciolukas      # Binario compilado
├── MANUAL_USUARIO.md
├── MANUAL_ADMIN.md
├── CLAUDE.md
└── LEEME.md
```

---

## Sistema de naming del ejecutable

El nombre del binario determina qué archivo de configuración se carga:

```
copy_directories_iniciolukas
                 ^^^^^^^^^^^^
                 tercer segmento (split por '_')
                 → busca inputs-backup/iniciolukas.json  (Linux)
                 → busca c:\xcopy\tools\iniciolukas.json (Windows)
```

Para agregar un nuevo perfil de usuario:
1. Crear `inputs-backup/inicioPERFIL.json`
2. Renombrar el binario a `copy_directories_inicioPERFIL`

---

## Archivos de configuración

### `iniciolukas.json`
```json
{
    "archivo_de_rutas": "./inputs-backup/backup_lukas_rutas.json",
    "archivo_excluir_carpetas": "./inputs-backup/backup_lukas_excluir_carpetas.txt",
    "apagar_equipo": "n"
}
```

- `apagar_equipo`: `"y"/"s"` → hibernar (Windows), `"off"` → apagar (Windows), `"n"` → nada

### `backup_lukas_rutas.json`
```json
{
    "rutas_origen_destino": [
        ["/origen/", "destino/relativo"]
    ],
    "destino_final": "/mnt/g",
    "name_disk": "BACKUP-LKS"
}
```

- `destino_final`: punto de montaje del disco de backup
- `name_disk`: etiqueta del disco (se valida al iniciar)
- `rutas_origen_destino`: pares `[origen, destino_relativo]`  
  El destino final = `destino_final/destino_relativo`

### `backup_lukas_excluir_carpetas.txt`
Un patrón por línea. Soporta comentarios con `#` y líneas vacías.  
Ejemplos válidos: `node_modules`, `*.log`, `.env.*.local`, `dist`

---

## Agregar una nueva ruta de backup

Editar `inputs-backup/backup_lukas_rutas.json` y agregar el par:
```json
["/home/lukasar/nueva_carpeta/", "linux/nueva_carpeta"]
```

---

## Compilar

```bash
# Desarrollo
cargo build

# Producción (recomendado para uso diario)
cargo build --release

# Ejecutar tests
cargo test

# Ejecutar test del disco real (requiere disco conectado)
cargo test -- --ignored
```

---

## Validación del disco de backup

Al iniciar, la app verifica en orden:
1. **Directorio existe** (`/mnt/g/`)
2. **Hay un disco montado** (compara device ID con el padre — `esta_montado()`)
3. **Tiene permisos de escritura** (crea archivo temporal de prueba)

En Windows además verifica que el nombre del disco coincida con `name_disk`.

---

## Montar el disco en Linux

```bash
# Montar manualmente
udisksctl mount -b /dev/sdb1

# El symlink /mnt/g apunta al montaje automático de GNOME
ls -la /mnt/g   # → /mnt/g -> /media/lukasar/BACKUP-LKS

# Desmontar correctamente
udisksctl unmount -b /dev/sdb1 && udisksctl power-off -b /dev/sdb
```

El automontaje de GNOME está habilitado:
```bash
gsettings get org.gnome.desktop.media-handling automount  # → true
```

---

## Logs

| Plataforma | Ubicación |
|-----------|-----------|
| Linux | `~/logs/backup/` |
| Windows | `C:\xcopy\` (resumen) y `C:\xcopy\logs\` (detalle) |

Formato del nombre: `YYYY-NMes-D_HHhorMMmin_log_backup.txt`

---

## Tests

```bash
cargo test                  # 57 tests unitarios
cargo test -- --ignored     # + test con disco real conectado
```

Módulos cubiertos: `platform`, `commands`, `config::types`, `tools::check_drive_backup`
