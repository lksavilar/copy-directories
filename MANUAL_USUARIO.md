# Manual de Usuario — Backup LKS

## ¿Qué hace esta aplicación?

Realiza copias de seguridad (backup) de tus carpetas al disco externo **BACKUP-LKS**, y permite restaurarlas cuando lo necesites. Funciona en **Linux** y **Windows** sin cambiar nada.

---

## Antes de ejecutar

1. **Conectar el disco externo** `BACKUP-LKS` al equipo.
2. Esperar a que el sistema lo monte automáticamente (aparece en el panel de Archivos).
3. Verificar que `/mnt/g` está activo:
   ```bash
   ls /mnt/g/linux/
   ```

> Si el disco no aparece, ejecutar: `udisksctl mount -b /dev/sdb1`

---

## Ejecutar el Backup

### Desde el escritorio
Doble clic en el ícono **Backup LKS** en el escritorio.  
Se abre una terminal y el proceso inicia automáticamente.

### Desde la terminal
```bash
cd ~/rust_projects/copy_directories
./target/release/copy_directories_iniciolukas backup
```

### ¿Qué se respalda?

| Carpeta origen | Destino en disco |
|---------------|-----------------|
| `~/rust_projects/` | `/mnt/g/linux/rust_projects` |
| `~/rust-dioxus-docker/` | `/mnt/g/linux/rust-dioxus-docker` |
| `~/dioxus-spa/` | `/mnt/g/linux/dioxus-spa` |
| `~/python_projects/` | `/mnt/g/linux/python_projects` |

### ¿Qué se excluye automáticamente?

`node_modules`, `target`, `.git`, `.venv`, `dist`, `.next`, `build`, archivos `.log`, `.env`, y otros archivos temporales. Ver lista completa en `inputs-backup/backup_lukas_excluir_carpetas.txt`.

---

## Restaurar archivos

```bash
# Restaurar todo (pide confirmación)
./target/release/copy_directories_iniciolukas restore

# Restaurar solo una carpeta específica
./target/release/copy_directories_iniciolukas restore --path "rust_projects"

# Restaurar sin confirmación
./target/release/copy_directories_iniciolukas restore --force
```

---

## Logs del backup

Los logs se guardan en `~/logs/backup/`:

| Archivo | Contenido |
|---------|-----------|
| `*_resumen_backup.txt` | Resumen: inicio, fin, estado |
| `*_log_backup.txt` | Detalle completo de archivos copiados |

---

## Desconectar el disco correctamente

**Siempre** expulsar antes de desconectar para evitar pérdida de datos:

```bash
udisksctl unmount -b /dev/sdb1 && udisksctl power-off -b /dev/sdb
```

O usar el botón **Expulsar** en la aplicación Archivos.

---

## Errores comunes

| Error | Causa | Solución |
|-------|-------|----------|
| `El directorio '/mnt/g/' no existe` | Disco no conectado | Conectar el disco |
| `no hay ningún disco montado` | Directorio existe pero disco no montado | `udisksctl mount -b /dev/sdb1` |
| `Sin permisos de escritura` | Permisos incorrectos | `sudo chown $USER:$USER /mnt/g` |
