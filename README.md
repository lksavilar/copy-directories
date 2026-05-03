# Backup Directories - Herramienta Multiplataforma

## 📋 Descripción

Aplicación Rust multiplataforma para realizar backup y restore de directorios usando herramientas nativas del sistema operativo:
- **Windows**: RoboCopy (máximo rendimiento)
- **Linux/macOS**: rsync (flexibilidad y compatibilidad)

## 🏗️ Estructura del Proyecto

```
backup_directories/
├── Cargo.toml                          # Configuración del proyecto Rust
├── src/
│   ├── main.rs                        # Punto de entrada principal
│   ├── cli.rs                         # Parser de argumentos CLI
│   ├── platform.rs                    # Detección de plataforma
│   ├── commands.rs                    # Comandos multiplataforma (RoboCopy/rsync)
│   ├── backup.rs                      # Lógica de backup
│   ├── restore.rs                     # Lógica de restore
│   ├── config/
│   │   ├── mod.rs                     # Módulo de configuración
│   │   └── types.rs                   # Tipos de datos (Config, Inicio, Fecha)
│   └── tools.rs                       # Utilidades compartidas
├── inputs/                            # ⭐ ARCHIVOS DE CONFIGURACIÓN
│   ├── iniciolukas.json               # Config principal Linux
│   ├── iniciolukas_windows.json       # Config principal Windows
│   ├── backup_lukas_rutas.json        # Rutas backup Linux
│   ├── backup_lukas_rutas_windows.json # Rutas backup Windows
│   └── backup_lukas_excluir_carpetas.txt # Carpetas/archivos a excluir
└── target/                            # Directorio de compilación (generado)
```

## 🔧 Sistema de Naming Automático

### Funcionamiento del Naming
El programa determina qué archivo de configuración usar basándose en **el nombre del ejecutable**:

```bash
Ejecutable: backup_directories_NOMBRE.exe
           ↓
Busca archivo: NOMBRE.json
```

### Ejemplos
```bash
# Para usuario Luis
backup_directories_inicioluis.exe    → busca inicioluis.json

# Para usuario Ana  
backup_directories_inicioana.exe     → busca inicioana.json

# Para trabajo
backup_directories_iniciotrabajo.exe → busca iniciotrabajo.json
```

### Algoritmo de Parsing
```rust
// Divide el nombre por '_' y toma el tercer elemento
"backup_directories_lukas" → ["backup", "directories", "lukas"] → "lukas.json"
```

## 📁 Archivos de Configuración

### 1. Archivo Principal (`NOMBRE.json`)

**Ubicación por plataforma:**
- **Windows**: `c:\xcopy\tools\NOMBRE.json`
- **Linux**: `.//NOMBRE.json`

**Estructura:**
```json
{
    "archivo_de_rutas": "./inputs/backup_lukas_rutas.json",
    "archivo_excluir_carpetas": "./inputs/backup_lukas_excluir_carpetas.txt",
    "apagar_equipo": "n"
}
```

**Campos:**
- `archivo_de_rutas`: Path al archivo de rutas origen/destino
- `archivo_excluir_carpetas`: Path al archivo de exclusiones
- `apagar_equipo`: Control de apagado Windows (`"y"`, `"s"`, `"off"`, `"n"`)

### 2. Archivo de Rutas (`backup_rutas.json`)

**Estructura:**
```json
{
    "rutas_origen_destino": [
        ["/ruta/origen1", "destino1"],
        ["/ruta/origen2", "destino2"],
        ["C:\\Users\\lucas\\projects", "C\\Users\\lucas\\projects"]
    ],
    "destino_final": "/mnt/backup/",
    "name_disk": "BACKUP-USB"
}
```

**Campos:**
- `rutas_origen_destino`: Array de pares `[origen, destino_relativo]`
- `destino_final`: Directorio base donde se almacenan los backups
- `name_disk`: Nombre esperado del disco/dispositivo de backup

**Construcción de rutas finales:**
```
Destino Final = destino_final + destino_relativo
Ejemplo: "/mnt/backup/" + "home/user/docs" = "/mnt/backup/home/user/docs"
```

### 3. Archivo de Exclusiones (`backup_excluir_carpetas.txt`)

**Formato**: Un directorio/archivo por línea
```text
node_modules
.vscode
.git
target
build
debug
.svelte-kit
System Volume Information
$RECYCLE.BIN
*.tmp
*.log
cache/
temp/
```

**Características:**
- Funciona en ambas plataformas
- Se convierte automáticamente al formato correcto:
  - **Windows RoboCopy**: `/XD node_modules .vscode .git...`
  - **Linux rsync**: `--exclude="node_modules" --exclude=".vscode"...`

## 🔒 Verificaciones de Seguridad

### Windows
1. **Extrae letra de drive** del `destino_final` (`G:\` → `G:`)
2. **Verifica que el drive existe** y está montado
3. **Compara nombre del disco** con `name_disk`
4. **Si no coincide**: Muestra error, lista drives disponibles y termina
5. **Si coincide**: Continúa con el backup

### Linux
1. **Verifica que el directorio existe** (`/mnt/backup/`)
2. **Prueba permisos de escritura** creando archivo temporal
3. **Si falla**: Sugiere comandos para crear/arreglar permisos y termina
4. **Si éxito**: Continúa con el backup

## 🚀 Compilación y Uso

### Compilación
```bash
# Desarrollo
cargo build

# Release optimizado
cargo build --release
```

### Nombrado del Ejecutable
**Importante**: El nombre en `Cargo.toml` determina el archivo de configuración:

```toml
[package]
name = "backup_directories_iniciolukas"  # → busca iniciolukas.json
```

### Uso

#### Modo Compatibilidad (sin argumentos)
```bash
# Ejecuta backup automáticamente
./backup_directories_lukas
backup_directories_lukas.exe
```

#### Modo CLI (con argumentos)
```bash
# Backup completo
./backup_directories_lukas backup

# Restore completo (con confirmación)
./backup_directories_lukas restore

# Restore específico (filtra por nombre de destino)
./backup_directories_lukas restore --path "rust_projects"

# Restore sin confirmación
./backup_directories_lukas restore --force

# Ayuda
./backup_directories_lukas help
```

## 📂 Ejemplos de Configuración

### Configuración Windows
```json
// iniciolukas_windows.json
{
    "archivo_de_rutas": "./inputs/backup_lukas_rutas_windows.json",
    "archivo_excluir_carpetas": "./inputs/backup_lukas_excluir_carpetas.txt",
    "apagar_equipo": "n"
}

// backup_lukas_rutas_windows.json
{
    "rutas_origen_destino": [
        ["C:\\Users\\lucas\\projects", "C\\Users\\lucas\\projects"],
        ["C:\\Users\\lucas\\Documents", "C\\Users\\lucas\\Documents"],
        ["D:\\", "D"]
    ],
    "destino_final": "G:\\",
    "name_disk": "BACKUP-LKS"
}
```

### Configuración Linux
```json
// iniciolukas.json
{
    "archivo_de_rutas": "./inputs/backup_lukas_rutas.json",
    "archivo_excluir_carpetas": "./inputs/backup_lukas_excluir_carpetas.txt",
    "apagar_equipo": "n"
}

// backup_lukas_rutas.json
{
    "rutas_origen_destino": [
        ["/home/lukasar/rust_projects", "home/lukasar/rust_projects"],
        ["/home/lukasar/Documents", "lks/Documents"],
        ["/home/lukasar/Downloads", "f2025/home/lukasar/Downloads"]
    ],
    "destino_final": "/mnt/e/",
    "name_disk": "ACER-LUKAS"
}
```

## 🔄 Lógica de Restore

El restore **invierte automáticamente** las rutas de backup:

| Fase | Origen | Destino |
|------|--------|---------|
| **Backup** | `/home/lukasar/rust_projects/` | `/mnt/g/wsl/rust_projects` |
| **Restore** | `/mnt/g/wsl/rust_projects` | `/home/lukasar/rust_projects/` |

**Proceso:**
1. Lee la misma configuración que el backup
2. Para cada ruta: `origen_backup = destino_final + destino_relativo`
3. Restaura hacia: `destino_restore = origen_original`

**Filtrado por --path:**
- El filtro busca en el **nombre de destino** (segunda columna de la configuración)
- `--path "rust_projects"` → Encuentra rutas que contengan "rust_projects" en el destino
- `--path "wsl"` → Encuentra todas las rutas (todas contienen "wsl")

**Ejemplos de filtrado:**
```bash
# Configuración actual:
# ["/home/lukasar/rust_projects/", "wsl/rust_projects"]
# ["/home/lukasar/dioxus-spa/", "wsl/dioxus-spa"]

./backup_directories_lukas restore --path "rust_projects"  # ✅ Restaura solo rust_projects
./backup_directories_lukas restore --path "dioxus"         # ✅ Restaura solo dioxus-spa  
./backup_directories_lukas restore --path "wsl"            # ✅ Restaura todo (ambas rutas)
./backup_directories_lukas restore --path "Documents"      # ❌ No encuentra nada
```

## 🎯 Características Avanzadas

### Detección Automática de Plataforma
```rust
// El programa detecta automáticamente y usa:
Windows → RoboCopy con parámetros optimizados
Linux   → rsync con configuración robusta
macOS   → rsync (compatible)
```

### Procesamiento de Salida
- **Windows RoboCopy**: Parsea con regex para mostrar archivos nuevos/modificados
- **Linux rsync**: Procesa símbolos de estado para mostrar transferencias

### Logs Automáticos
```
Windows: C:/xcopy/logs/YYYY-MM-DD_HHhorMMmin_log_backup.txt
Linux:   /tmp/YYYY-MM-DD_HHhorMMmin_log_backup.txt
```

## 🛠️ Desarrollo

### Dependencias
```toml
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1"
winapi = { version = "0.3", features = ["fileapi"] }  # Solo Windows
sysinfo = "0.28"
clap = { version = "4.0", features = ["derive"] }
```

### Módulos Clave
- **platform.rs**: Detección OS y normalización de rutas
- **commands.rs**: Abstracción RoboCopy/rsync
- **tools.rs**: Verificación de drives/directorios
- **config/**: Carga y validación de configuraciones

### Testing
```bash
# Compilar y probar
cargo build
cargo run -- backup
cargo run -- restore --force
```

## 📄 Licencia

Este proyecto mantiene la funcionalidad y filosofía del código original, extendido para soporte multiplataforma.

---
**Generado con ❤️ para backup seguro y confiable**