# Manual de Usuario — Backup LKS

## ¿Qué hace esta aplicación?

Realiza copias de seguridad (backup) de tus carpetas al disco externo **BACKUP-LKS**, y permite restaurarlas cuando lo necesites. Funciona en **Linux** y **Windows** sin cambiar nada.

---

## LINUX

### Antes de ejecutar

1. Conectar el disco externo **BACKUP-LKS** al equipo.
2. Esperar a que GNOME lo monte automáticamente (aparece en el panel de Archivos).
3. Verificar que el disco está accesible:
   ```bash
   ls /mnt/g/linux/
   ```

> Si el disco no aparece, montarlo manualmente:
> ```bash
> udisksctl mount -b /dev/sdb1
> ```

### Ejecutar el Backup

**Desde el escritorio:** doble clic en el ícono **Backup LKS**.

**Desde la terminal:**
```bash
cd ~/rust_projects/copy_directories
./target/release/copy_directories_iniciolukas backup
```

### ¿Qué se respalda? (Linux)

| Carpeta origen | Destino en disco |
|---------------|-----------------|
| `~/rust_projects/` | `BACKUP-LKS/linux/rust_projects` |
| `~/rust-dioxus-docker/` | `BACKUP-LKS/linux/rust-dioxus-docker` |
| `~/dioxus-spa/` | `BACKUP-LKS/linux/dioxus-spa` |
| `~/python_projects/` | `BACKUP-LKS/linux/python_projects` |

### Logs (Linux)

Los logs se guardan en `~/logs/backup/`:

| Archivo | Contenido |
|---------|-----------|
| `*_resumen_backup.txt` | Resumen: inicio, fin, estado |
| `*_log_backup.txt` | Detalle completo de archivos copiados |

### Desconectar el disco (Linux)

**Siempre** expulsar antes de desconectar para evitar pérdida de datos:

```bash
udisksctl unmount -b /dev/sdb1 && udisksctl power-off -b /dev/sdb
```

O usar el botón **Expulsar** en la aplicación Archivos.

### Errores comunes (Linux)

| Error | Causa | Solución |
|-------|-------|----------|
| `El directorio '/mnt/g/' no existe` | Disco no conectado | Conectar el disco |
| `no hay ningún disco montado` | Directorio existe pero disco no montado | `udisksctl mount -b /dev/sdb1` |
| `Sin permisos de escritura` | Permisos incorrectos | `sudo chown $USER:$USER /mnt/g` |

---

## WINDOWS

### Antes de ejecutar

1. Conectar el disco externo **BACKUP-LKS** al equipo.
2. Windows lo monta automáticamente como unidad **G:**.
3. Verificar en el Explorador de archivos que aparece `G:` con el nombre **BACKUP-LKS**.

> Si Windows no lo asigna a G:, ir a **Administración de discos** y cambiar la letra de unidad a `G`.

### Ejecutar el Backup

**Desde el Explorador:** doble clic en `copy_directories_iniciolukas.exe`.

**Desde CMD o PowerShell:**
```cmd
cd C:\ruta\al\proyecto
copy_directories_iniciolukas.exe backup
```

### ¿Qué se respalda? (Windows)

| Carpeta origen | Destino en disco |
|---------------|-----------------|
| `C:\Users\lucas\projects` | `BACKUP-LKS\C\Users\lucas\projects` |
| `C:\Users\lucas\source` | `BACKUP-LKS\C\Users\lucas\source` |
| `C:\Users\lucas\Documents` | `BACKUP-LKS\C\Users\lucas\Documents` |
| `C:\Users\lucas\rust-lang` | `BACKUP-LKS\C\Users\lucas\rust-lang` |
| `C:\Users\lucas\haskell-lang` | `BACKUP-LKS\C\Users\lucas\haskell-lang` |
| `C:\Users\lucas\elm-lang` | `BACKUP-LKS\C\Users\lucas\elm-lang` |
| `C:\Users\lucas\cobol-lang` | `BACKUP-LKS\C\Users\lucas\cobol-lang` |
| `C:\Users\lucas\PycharmProjects` | `BACKUP-LKS\C\Users\lucas\PycharmProjects` |
| `C:\xcopy` | `BACKUP-LKS\C\xcopy` |
| `D:\` | `BACKUP-LKS\D` |

### Apagado automático (Windows)

Se puede configurar en `inputs-backup/iniciolukas_windows.json`:

| Valor `apagar_equipo` | Acción al terminar |
|----------------------|--------------------|
| `"n"` | No hace nada (por defecto) |
| `"y"` o `"s"` | Hiberna el equipo |
| `"off"` | Apaga el equipo |

### Logs (Windows)

| Ubicación | Contenido |
|-----------|-----------|
| `C:\xcopy\*_resumen_backup.txt` | Resumen: inicio, fin, estado |
| `C:\xcopy\logs\*_log_backup.txt` | Detalle completo de archivos copiados |

### Desconectar el disco (Windows)

Usar el botón **Expulsar** en la barra de tareas (ícono de USB) antes de desconectar físicamente el disco.

### Errores comunes (Windows)

| Error | Causa | Solución |
|-------|-------|----------|
| `Drive G: no encontrado` | Disco no conectado o letra incorrecta | Conectar el disco; asignar letra G en Administración de discos |
| `Nombre del disco no coincide` | Disco diferente en G: | Verificar que el disco conectado es **BACKUP-LKS** |

---

## Restaurar archivos (ambas plataformas)

```bash
# Restaurar todo (pide confirmación)
./copy_directories_iniciolukas restore

# Restaurar solo una carpeta específica
./copy_directories_iniciolukas restore --path "rust_projects"

# Restaurar sin confirmación
./copy_directories_iniciolukas restore --force
```

El restore **invierte las rutas**: copia desde el disco de backup de vuelta a las carpetas originales.

---

## ¿Qué se excluye automáticamente?

`node_modules`, `target`, `.git`, `.venv`, `dist`, `.next`, `build`, archivos `.log`, `.env`, y otros archivos temporales. Ver lista completa en `inputs-backup/backup_lukas_excluir_carpetas.txt`.
