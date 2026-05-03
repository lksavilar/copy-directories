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
│   └── backup_lukas_excluir_carpetas.txt # Patrones a excluir
├── target/release/
│   └── copy_directories_iniciolukas      # Binario compilado (Linux)
├── MANUAL_USUARIO.md
├── MANUAL_ADMIN.md
└── CLAUDE.md
```

---

## Sistema de naming del ejecutable

El nombre del binario determina qué archivo de configuración se carga:

```
copy_directories_iniciolukas
                 ^^^^^^^^^^^^
                 tercer segmento (split por '_')
                 → Linux:   ./inputs-backup/iniciolukas.json
                 → Windows: C:\xcopy\tools\iniciolukas.json
```

Para agregar un nuevo perfil:
1. Crear el JSON de inicio correspondiente (`inicioPERFIL.json`)
2. Renombrar el binario a `copy_directories_inicioPERFIL`

---

## Archivo de exclusiones

`inputs-backup/backup_lukas_excluir_carpetas.txt` — compartido entre Linux y Windows.  
Un patrón por línea. Soporta comentarios con `#` y líneas vacías.  
Ejemplos válidos: `node_modules`, `*.log`, `.env.*.local`, `dist`

---

## Tests

```bash
cargo test                  # 57 tests unitarios (sin disco)
cargo test -- --ignored     # + 1 test con disco BACKUP-LKS montado en /mnt/g/
```

Módulos cubiertos: `platform`, `commands`, `config::types`, `tools::check_drive_backup`

---

---

## LINUX

### Compilar

```bash
cargo build --release
# Binario generado: ./target/release/copy_directories_iniciolukas
```

### Archivo de configuración principal

`inputs-backup/iniciolukas.json`:
```json
{
    "archivo_de_rutas": "./inputs-backup/backup_lukas_rutas.json",
    "archivo_excluir_carpetas": "./inputs-backup/backup_lukas_excluir_carpetas.txt",
    "apagar_equipo": "n"
}
```

### Archivo de rutas

`inputs-backup/backup_lukas_rutas.json`:
```json
{
    "rutas_origen_destino": [
        ["/home/lukasar/rust_projects/", "linux/rust_projects"],
        ["/home/lukasar/rust-dioxus-docker/", "linux/rust-dioxus-docker"],
        ["/home/lukasar/dioxus-spa/", "linux/dioxus-spa"],
        ["/home/lukasar/python_projects/", "linux/python_projects"]
    ],
    "destino_final": "/mnt/g/",
    "name_disk": "BACKUP-LKS"
}
```

- `destino_final`: punto de montaje del disco de backup
- `name_disk`: etiqueta del disco (referencial en Linux)
- `rutas_origen_destino`: pares `[origen_absoluto, destino_relativo]`  
  Destino real = `destino_final` + `destino_relativo`

### Agregar una nueva ruta de backup (Linux)

Editar `inputs-backup/backup_lukas_rutas.json` y agregar:
```json
["/home/lukasar/nueva_carpeta/", "linux/nueva_carpeta"]
```

### Configuración del punto de montaje `/mnt/g`

`/mnt/g` es un symlink que apunta al montaje automático de GNOME:

```bash
# Verificar symlink
ls -la /mnt/g   # → /mnt/g -> /media/lukasar/BACKUP-LKS

# Crear el symlink (solo si no existe)
sudo ln -sf /media/lukasar/BACKUP-LKS /mnt/g

# Verificar que el automontaje de GNOME está activo
gsettings get org.gnome.desktop.media-handling automount  # → true

# Activarlo si estuviera desactivado
gsettings set org.gnome.desktop.media-handling automount true
```

### Montar y desmontar el disco (Linux)

```bash
# Montar manualmente (si GNOME no lo montó automáticamente)
udisksctl mount -b /dev/sdb1

# Verificar montaje
lsblk -o NAME,LABEL,MOUNTPOINT

# Desmontar correctamente antes de desconectar
udisksctl unmount -b /dev/sdb1 && udisksctl power-off -b /dev/sdb
```

### Validación del disco al arrancar (Linux)

`tools::check_drive_backup` verifica en orden:
1. **Directorio existe** — `Path::new("/mnt/g/").exists()`
2. **Hay disco montado** — `esta_montado()` compara `dev()` del path con su padre (device ID distinto = disco montado)
3. **Permisos de escritura** — crea y elimina un archivo temporal de prueba

Si falla cualquier paso, la app imprime el error, sugiere el comando correcto y termina.

### Logs (Linux)

Ubicación: `~/logs/backup/`  
El directorio se crea automáticamente si no existe.

| Archivo | Contenido |
|---------|-----------|
| `YYYY-NMes-D_HHhorMMmin_resumen_backup.txt` | Inicio, fin, estado general |
| `YYYY-NMes-D_HHhorMMmin_log_backup.txt` | Detalle completo de archivos copiados |

### Acceso de escritorio (Linux)

El archivo `~/Escritorio/backup-lks.desktop` lanza el backup con doble clic:

```ini
[Desktop Entry]
Name=Backup LKS
Exec=gnome-terminal -- bash -c "cd /home/lukasar/rust_projects/copy_directories && ./target/release/copy_directories_iniciolukas backup; exec bash"
Icon=/home/lukasar/rust_projects/copy_directories/icono-hombre.png
Terminal=false
Type=Application
```

Para que el ícono sea clicable en GNOME, marcar el archivo como de confianza:
```bash
gio set ~/Escritorio/backup-lks.desktop metadata::trusted true
chmod +x ~/Escritorio/backup-lks.desktop
```

---

---

## WINDOWS

### Compilar

Compilar en una máquina Windows con Rust instalado:

```cmd
cargo build --release
:: Binario generado: target\release\copy_directories_iniciolukas.exe
```

O hacer cross-compilación desde Linux (requiere toolchain Windows):
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

### Ubicación del archivo de configuración principal

En Windows, la app busca el JSON de inicio en una ruta fija:

```
C:\xcopy\tools\iniciolukas.json
```

Este archivo **no** se lee desde la carpeta del proyecto — debe copiarse a esa ubicación.

`C:\xcopy\tools\iniciolukas.json`:
```json
{
    "archivo_de_rutas": "./inputs-backup/backup_lukas_rutas_windows.json",
    "archivo_excluir_carpetas": "./inputs-backup/backup_lukas_excluir_carpetas.txt",
    "apagar_equipo": "n"
}
```

> Las rutas relativas (`archivo_de_rutas`, `archivo_excluir_carpetas`) se resuelven desde el **directorio de trabajo** donde se ejecuta el `.exe`, no desde `C:\xcopy\tools\`.

### Archivo de rutas

`inputs-backup/backup_lukas_rutas_windows.json`:
```json
{
    "rutas_origen_destino": [
        ["C:\\Users\\lucas\\projects",        "C\\Users\\lucas\\projects"],
        ["C:\\Users\\lucas\\source",          "C\\Users\\lucas\\source"],
        ["C:\\Users\\lucas\\Documents",       "C\\Users\\lucas\\Documents"],
        ["C:\\Users\\lucas\\rust-lang",       "C\\Users\\lucas\\rust-lang"],
        ["C:\\Users\\lucas\\haskell-lang",    "C\\Users\\lucas\\haskell-lang"],
        ["C:\\Users\\lucas\\elm-lang",        "C\\Users\\lucas\\elm-lang"],
        ["C:\\Users\\lucas\\cobol-lang",      "C\\Users\\lucas\\cobol-lang"],
        ["C:\\Users\\lucas\\PycharmProjects", "C\\Users\\lucas\\PycharmProjects"],
        ["C:\\xcopy",                         "C\\xcopy"],
        ["D:\\",                              "D"]
    ],
    "destino_final": "G:\\",
    "name_disk": "BACKUP-LKS"
}
```

### Agregar una nueva ruta de backup (Windows)

Editar `inputs-backup/backup_lukas_rutas_windows.json` y agregar:
```json
["C:\\Users\\lucas\\nueva_carpeta", "C\\Users\\lucas\\nueva_carpeta"]
```

### Validación del disco al arrancar (Windows)

`tools::check_drive_backup` verifica en orden:
1. **Extrae la letra de drive** de `destino_final` (`"G:\\"` → `"G:\\"`)
2. **Verifica que el drive existe** — `fs::metadata("G:\\").is_ok()`
3. **Compara el nombre del disco** con `name_disk` usando `sysinfo`

Si el disco en G: no se llama `BACKUP-LKS`, la app lista todos los drives montados y termina sin hacer backup.

### Apagado automático post-backup (Windows)

Configurar en `C:\xcopy\tools\iniciolukas.json`:

| Valor `apagar_equipo` | Acción |
|----------------------|--------|
| `"n"` | No hace nada (por defecto) |
| `"y"` o `"s"` | Hiberna (`shutdown /h`) |
| `"off"` | Apaga (`shutdown /s /t 0`) |

### Logs (Windows)

| Archivo | Contenido |
|---------|-----------|
| `C:\xcopy\YYYY-NMes-D_HHhorMMmin_resumen_backup.txt` | Inicio, fin, estado general |
| `C:\xcopy\logs\YYYY-NMes-D_HHhorMMmin_log_backup.txt` | Detalle completo de archivos copiados |

El directorio `C:\xcopy\logs\` debe existir antes de ejecutar el backup por primera vez:
```cmd
mkdir C:\xcopy\logs
```

### Funciones críticas — no modificar sin entender

| Función | Qué hace |
|---------|----------|
| `extraer_letra_drive()` | Parsea la letra de drive Windows desde `destino_final` |
| `esta_montado()` | Usa `MetadataExt::dev()` — solo compila en Linux/macOS |
| Códigos éxito robocopy | **0–7 = éxito**, 8+ = error (no cambiar este rango) |
