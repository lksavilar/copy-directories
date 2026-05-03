pub mod check_drive_backup {
    use std::fs;
    use sysinfo::{DiskExt, System, SystemExt};
    use std::io::{self, Write};

    pub fn check_drive_backup(init: &crate::config::Inicio, config: &crate::config::Config, file_rs: &mut dyn Write) -> io::Result<()> {
        use crate::platform::{detect_platform, Platform};

        let platform = detect_platform();

        match platform {
            Platform::Windows => check_windows_drive(init, config, file_rs),
            Platform::Linux | Platform::MacOS => check_linux_mount(config, file_rs),
            Platform::Unknown => {
                println!("Plataforma no soportada para verificación de backup");
                Ok(())
            }
        }
    }

    /// Verifica que el directorio existe y tiene permisos de escritura.
    /// Retorna Ok(()) si está listo, o Err con descripción del problema.
    fn validar_directorio(backup_path: &str) -> Result<(), String> {
        use std::path::Path;

        if !Path::new(backup_path).exists() {
            return Err(format!("El directorio de BACKUP '{}' no existe", backup_path));
        }

        let test_file = format!("{}/test_write_permission", backup_path.trim_end_matches('/'));
        match fs::File::create(&test_file) {
            Ok(_) => {
                let _ = fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => Err(format!("Sin permisos de escritura en '{}': {}", backup_path, e)),
        }
    }

    /// Extrae la letra de drive de una ruta Windows ("G:\\backup" → Some("G:\\")).
    /// Retorna None si la ruta no contiene ':' (ruta Linux/macOS).
    fn extraer_letra_drive(destino_final: &str) -> Option<String> {
        if destino_final.contains(':') {
            let letra = destino_final.splitn(2, ':').next()?;
            Some(format!("{}:\\", letra))
        } else {
            None
        }
    }

    /// Verifica que hay un filesystem independiente montado en `path`.
    /// Compara el device ID del path con el de su directorio padre:
    /// si son distintos, hay un disco/volumen montado ahí.
    #[cfg(not(target_os = "windows"))]
    fn esta_montado(path: &str) -> bool {
        use std::os::unix::fs::MetadataExt;
        use std::path::Path;

        let p = Path::new(path);
        let parent = p.parent().unwrap_or(p);

        match (fs::metadata(p), fs::metadata(parent)) {
            (Ok(meta), Ok(parent_meta)) => meta.dev() != parent_meta.dev(),
            _ => false,
        }
    }

    fn check_windows_drive(init: &crate::config::Inicio, config: &crate::config::Config, file_rs: &mut dyn Write) -> io::Result<()> {
        let path = match extraer_letra_drive(&config.destino_final) {
            Some(p) => p,
            None => {
                println!("No se pudo determinar la letra del drive desde '{}'", config.destino_final);
                writeln!(file_rs, "No se pudo determinar la letra del drive desde '{}'", config.destino_final)?;
                super::utils::pausa("Presiona Enter para salir...\n");
                std::process::exit(0);
            }
        };

        if fs::metadata(&path).is_ok() {
            let mut sys = System::new_all();
            sys.refresh_disks_list();

            for disk in sys.disks() {
                if let Some(mount_point) = disk.mount_point().to_str() {
                    if mount_point == &path {
                        let namedisk = disk.name().to_string_lossy();
                        if config.name_disk != namedisk {
                            let subcadena: String = mount_point.chars().take(2).collect();
                            println!("\nVerificar el disco de BACKUP montado. No es el correcto.");
                            println!("\nRevisar drive montado por el sistema, este debe aparecer como:\n\t{} ({})", config.name_disk, subcadena);
                            writeln!(file_rs, "\nVerificar:\nEl disco de BACKUP montado en {}. No es el correcto.", mount_point)?;
                            writeln!(file_rs, "\nRevisar drive montado por el sistema, este debe aparecer como:\n\t{} ({})", config.name_disk, subcadena)?;
                            super::utils::pausa("Presiona Enter para salir...\n");
                            std::process::exit(0);
                        }
                        break;
                    }
                }
            }
            Ok(())
        } else {
            println!("La unidad de BACKUP {} NO está disponible.", path);
            println!("\nSe recomienda:\n-. Verificar que la unidad de BACKUP esté reconocida por el sistema.");
            println!("-. Si la unidad {} no es la de BACKUP, modificar el archivo \"{}\"", path, init.archivo_de_rutas);
            writeln!(file_rs, "\nLa unidad de BACKUP {} NO está disponible.", path)?;
            writeln!(file_rs, "\nSe recomienda:\n-. Verificar que la unidad de BACKUP esté reconocida por el sistema.")?;
            writeln!(file_rs, "-. Si la unidad {} no es la de BACKUP, modificar el archivo \"{}\"", path, init.archivo_de_rutas)?;

            match super::check_drives_mounted_on_the_system::misdrives(file_rs) {
                Ok((io_res, datos_res)) => {
                    if let Err(e) = io_res { eprintln!("Error de IO: {:?}", e); }
                    if let Err(e) = datos_res { eprintln!("Error en datos: {:?}", e); }
                }
                Err(e) => eprintln!("Error en la función principal: {}", e),
            }
            super::utils::pausa("Presiona Enter para salir...\n");
            std::process::exit(0);
        }
    }

    fn check_linux_mount(config: &crate::config::Config, file_rs: &mut dyn Write) -> io::Result<()> {
        let backup_path = &config.destino_final;

        // Paso 1: verificar que el directorio existe y es escribible
        if let Err(msg) = validar_directorio(backup_path) {
            println!("❌ {}", msg);
            writeln!(file_rs, "❌ {}", msg)?;
            if msg.contains("no existe") {
                println!("\nSe recomienda:");
                println!("-. Crear el directorio de backup: sudo mkdir -p {}", backup_path);
                println!("-. Asegurar permisos adecuados: sudo chown $USER:$USER {}", backup_path);
                writeln!(file_rs, "\nSe recomienda:\n-. Crear el directorio de backup: sudo mkdir -p {}", backup_path)?;
                writeln!(file_rs, "-. Asegurar permisos adecuados: sudo chown $USER:$USER {}", backup_path)?;
            } else {
                println!("\nSe recomienda:");
                println!("-. Verificar permisos: ls -la {}", backup_path);
                println!("-. Corregir permisos: sudo chown $USER:$USER {}", backup_path);
                writeln!(file_rs, "\nSe recomienda:\n-. Verificar permisos: ls -la {}", backup_path)?;
                writeln!(file_rs, "-. Corregir permisos: sudo chown $USER:$USER {}", backup_path)?;
            }
            super::utils::pausa("Presiona Enter para salir...\n");
            std::process::exit(0);
        }

        // Paso 2: verificar que hay un disco realmente montado (no solo directorio vacío)
        #[cfg(not(target_os = "windows"))]
        if !esta_montado(backup_path) {
            let msg = format!(
                "El directorio '{}' existe pero no hay ningún disco montado en él",
                backup_path
            );
            println!("❌ {}", msg);
            writeln!(file_rs, "❌ {}", msg)?;
            println!("\nSe recomienda:");
            println!("-. Conectar el disco de backup '{}'", config.name_disk);
            println!("-. Verificar el montaje: lsblk -o NAME,LABEL,MOUNTPOINT");
            writeln!(file_rs, "\nSe recomienda:\n-. Conectar el disco de backup '{}'", config.name_disk)?;
            writeln!(file_rs, "-. Verificar el montaje: lsblk -o NAME,LABEL,MOUNTPOINT")?;
            super::utils::pausa("Presiona Enter para salir...\n");
            std::process::exit(0);
        }

        println!("✅ Directorio de backup verificado: {}", backup_path);
        writeln!(file_rs, "✅ Directorio de backup verificado: {}", backup_path)?;

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // --- validar_directorio ---

        #[test]
        fn test_directorio_temp_existe_y_es_escribible() {
            let dir = std::env::temp_dir();
            let path = dir.to_string_lossy().to_string();
            let result = validar_directorio(&path);
            assert!(result.is_ok(), "El directorio temporal debe ser válido: {:?}", result);
        }

        #[test]
        fn test_directorio_inexistente_retorna_error() {
            let result = validar_directorio("/ruta/absolutamente/inexistente/xyz987abc");
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("no existe"), "El mensaje debe indicar que no existe: {}", msg);
        }

        #[test]
        fn test_directorio_inexistente_incluye_ruta_en_error() {
            let ruta = "/ruta/que/no/existe/abc123";
            let result = validar_directorio(ruta);
            assert!(result.is_err());
            assert!(
                result.unwrap_err().contains(ruta),
                "El mensaje de error debe incluir la ruta problemática"
            );
        }

        #[test]
        fn test_directorio_raiz_es_valido() {
            // La raíz siempre existe en Linux/macOS/Windows
            #[cfg(not(target_os = "windows"))]
            let result = validar_directorio("/tmp");
            #[cfg(target_os = "windows")]
            let result = validar_directorio("C:\\Windows\\Temp");
            assert!(result.is_ok());
        }

        // --- esta_montado (solo Linux/macOS) ---

        #[test]
        #[cfg(not(target_os = "windows"))]
        fn test_proc_es_punto_de_montaje() {
            // /proc siempre está montado como procfs en Linux — dev distinto a /
            assert!(esta_montado("/proc"), "/proc siempre es un punto de montaje en Linux");
        }

        #[test]
        #[cfg(not(target_os = "windows"))]
        fn test_subdirectorio_temporal_no_es_punto_de_montaje() {
            // Un subdirectorio creado manualmente tiene el mismo dev que su padre
            let subdir = std::env::temp_dir().join("test_mount_check_xyz_abc");
            fs::create_dir_all(&subdir).unwrap();
            let result = esta_montado(subdir.to_str().unwrap());
            let _ = fs::remove_dir(&subdir);
            assert!(!result, "Un subdirectorio normal no es punto de montaje");
        }

        #[test]
        #[cfg(not(target_os = "windows"))]
        fn test_path_inexistente_no_es_punto_de_montaje() {
            // fs::metadata falla → retorna false
            assert!(!esta_montado("/ruta/inexistente/xyz789abc"));
        }

        #[test]
        #[ignore = "Requiere disco BACKUP-LKS montado en /mnt/g/"]
        fn test_backup_drive_esta_montado() {
            assert!(
                esta_montado("/mnt/g/"),
                "El disco BACKUP-LKS debe estar montado en /mnt/g/"
            );
        }

        // --- extraer_letra_drive ---

        #[test]
        fn test_extrae_letra_g_de_ruta_windows() {
            assert_eq!(extraer_letra_drive("G:\\backup\\datos"), Some("G:\\".to_string()));
        }

        #[test]
        fn test_extrae_letra_c_de_ruta_windows() {
            assert_eq!(extraer_letra_drive("C:\\xcopy\\tools"), Some("C:\\".to_string()));
        }

        #[test]
        fn test_extrae_letra_d_de_ruta_windows() {
            assert_eq!(extraer_letra_drive("D:\\proyectos"), Some("D:\\".to_string()));
        }

        #[test]
        fn test_multiples_letras_de_drive() {
            for (input, expected) in [
                ("G:\\datos",     "G:\\"),
                ("D:\\proyectos", "D:\\"),
                ("E:\\backup",    "E:\\"),
                ("F:\\musica",    "F:\\"),
            ] {
                let result = extraer_letra_drive(input);
                assert_eq!(
                    result,
                    Some(expected.to_string()),
                    "Falló para entrada: {}", input
                );
            }
        }

        #[test]
        fn test_ruta_linux_sin_dos_puntos_retorna_none() {
            assert!(
                extraer_letra_drive("/mnt/g/backup").is_none(),
                "Ruta Linux no tiene letra de drive"
            );
        }

        #[test]
        fn test_ruta_linux_con_slash_retorna_none() {
            assert!(extraer_letra_drive("/home/user/docs").is_none());
        }

        #[test]
        fn test_cadena_vacia_retorna_none() {
            assert!(extraer_letra_drive("").is_none());
        }

        #[test]
        fn test_solo_letra_y_dos_puntos_es_valido() {
            assert_eq!(extraer_letra_drive("G:"), Some("G:\\".to_string()));
        }
    }
}

pub mod structs {
    use chrono::prelude::*;
    use chrono::{DateTime, Local}; // Asegúrate de tener en las dependencias: chrono = "0.4"
    use serde::Serialize;
    use std::fmt;

    #[derive(Serialize, Clone, Debug)]
    pub struct Fecha {
        dia_nombre: String,
        dia_numero: u32,
        mes_nombre: String,
        mes_numero: u32,
        anio: i32,
        hora_min_seg: String,
        hora24: u32,
        minutos: u32,
        segundos: u32,
        fecha_formateada: String,
    }
    impl Fecha {
        // Método para obtener la fecha formateada
        pub fn get_fecha_formateada(&self) -> &String {
            &self.fecha_formateada
        }

        // Método para obtener el nombre del día
        pub fn _get_dia_nombre(&self) -> &String {
            &self.dia_nombre
        }

        // Método para obtener el nombre del mes
        pub fn _get_mes_nombre(&self) -> &String {
            &self.mes_nombre
        }

        // Método para obtener la hora, minutos y segundos
        pub fn _get_hora_min_seg(&self) -> &String {
            &self.hora_min_seg
        }

        // Método para obtener día número
        pub fn get_dia_numero(&self) -> &u32 {
            &self.dia_numero
        }

        // Método para obtener mes número
        pub fn get_mes_numero(&self) -> &u32 {
            &self.mes_numero
        }

        // Método para obtener el año
        pub fn get_anio(&self) -> &i32 {
            &self.anio
        }

        // Método para obtener hora24
        pub fn _get_hora12(&self) -> &u32 {
            &self.hora24
        }

        // Método para obtener minutos
        pub fn _get_minutos(&self) -> &u32 {
            &self.minutos
        }

        // Método para obtener segundos
        pub fn _get_segundos(&self) -> &u32 {
            &self.segundos
        }

        // Método para obtener los primeros 3 caracteres de `mes_nombre` en minuscula
        pub fn _get_mes_nombre_corto(&self) -> String {
            self.mes_nombre.chars().take(3).collect()
        }
    }

    impl fmt::Display for Fecha {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.fecha_formateada)
        }
    }
    impl Fecha {
        // Método para obtener los primeros 3 caracteres de `mes_nombre` en mayúsculas
        pub fn get_mes_nombre3letras(&self) -> String {
            self.mes_nombre.to_uppercase().chars().take(3).collect()
        }
    }
    
    pub fn fecha() -> Fecha {

        // Estructura inicial de la fecha con valores por defecto
        let fecha_inicial = Fecha {
            dia_nombre: "".to_string(),
            dia_numero: 0,
            mes_nombre: "".to_string(),
            mes_numero: 0,
            anio: 0,
            hora_min_seg: "".to_string(),
            hora24: 0,
            minutos: 0,
            segundos: 0,
            fecha_formateada: "".to_string(),
        };

        // Actualizar la estructura con la fecha y hora actuales
        let fechaok = obtener_fecha(fecha_inicial.clone());
        // Convertir a JSON
        // let json = to_string(&fecha_actualizada).unwrap();
        //println!("{}", json);
        fechaok
    }

    // Función que actualiza la estructura de fecha
    fn obtener_fecha(mut fecha: Fecha) -> Fecha {
        let now: DateTime<Local> = Local::now();

        // Listas para días y meses en español
        let dias_semana = [
            "Domingo",
            "Lunes",
            "Martes",
            "Miércoles",
            "Jueves",
            "Viernes",
            "Sábado",
        ];
        let meses = [
            "Enero",
            "Febrero",
            "Marzo",
            "Abril",
            "Mayo",
            "Junio",
            "Julio",
            "Agosto",
            "Septiembre",
            "Octubre",
            "Noviembre",
            "Diciembre",
        ];

        // Actualizar los campos de la estructura
        fecha.dia_nombre = dias_semana[now.weekday().num_days_from_sunday() as usize].to_string();
        fecha.dia_numero = now.day();
        fecha.mes_nombre = meses[(now.month() - 1) as usize].to_string();
        fecha.mes_numero = now.month();
        fecha.anio = now.year();
        fecha.hora_min_seg = now.format("%H:%M:%S").to_string(); // Formato de hora: HH:MM:SS

        fecha.hora24 = now.hour();
        fecha.minutos = now.minute();
        fecha.segundos = now.second();

        // Actualizar el campo de la fecha formateada
        fecha.fecha_formateada = format!(
            "{}, {} de {} de {} {}",
            fecha.dia_nombre, fecha.dia_numero, fecha.mes_nombre, fecha.anio, fecha.hora_min_seg
        );

        fecha // Retornar la estructura actualizada
    }
}

// Verifica cuales drives están montados en el sistema
pub mod check_drives_mounted_on_the_system {
    use sysinfo::{DiskExt, System, SystemExt};
    use std::io;
    use std::io::Write;

    // Definir el tipo de retorno como una tupla en Result
    pub fn misdrives(file_rs: &mut dyn Write) -> Result<(io::Result<()>, Result<Vec<(String, String)>, String>), String> {
        // Crea una instancia del sistema
        let mut system = System::new_all();

        // Refresca la información del sistema para asegurarse de que está actualizada
        system.refresh_all();

        // Crea un vector para almacenar los nombres de las unidades (puntos de montaje)
        let mut drives: Vec<(String, String)> = Vec::new();
        // Crea un vector para almacenar los nombres de las unidades ( nombre del dispositivo)
        let _name_disk: Vec<String> = Vec::new();

        println!("\nUnidades encontradas en el sistema:");
        println!("-----------------------------------");
        writeln!(file_rs,"\nUnidades encontradas en el sistema:").unwrap();
        writeln!(file_rs,"-----------------------------------").unwrap();
        // Itera sobre los discos disponibles
        for disk in system.disks() {
            let disk_name = disk.name().to_string_lossy().into_owned();
            let mount_point = disk.mount_point().to_string_lossy().into_owned();

            // Muestra el nombre del disco
            println!("Unidad: {} Nombre del disco: {}", mount_point, disk_name);
            writeln!(file_rs,"Unidad: {} Nombre del disco: {}", mount_point, disk_name).unwrap();
            // Agrega el punto de montaje (como C:, D:, etc.) al vector de drives
            drives.push((disk_name, mount_point));
        }
        Ok((Ok(()),Ok(drives)))
    }

}

pub mod config_loader {
    use std::fs;    
    use std::env;
    use std::process;
    use crate::config::{Inicio, Config};
    use crate::tools::utils::pausa;

    pub fn program_name() -> Vec<String> {
        // Obtiene el nombre del programa .exe en ejecución
        let exe_path = env::current_exe().unwrap();
        let file_stem = exe_path.file_stem().unwrap();  // OsStr
        let name = file_stem.to_str().unwrap();         // &str

        name.split(|c| c == '*' || c == '_')
            .map(|s| s.to_string())                     // convierte &str → String
            .collect::<Vec<String>>()  
    }

    pub fn cargar_configuracion() -> (Inicio, Config, String) {
        // Conocer la tercera parte del nombre del archivo .exe, para saber que archivo de inicio?.json debe abrir.
        let partes_name = program_name();     // Vec<String>
        //println!("{:?}", partes_name );

        let tercer_elemento;
        if let Some(tercero) = partes_name.get(2) {
            tercer_elemento = tercero;
            println!("El nombre del archivo de inicio debera ser: {}.json", tercero);
        } else {
            println!("No hay un tercer elemento, mal conformado el nombre del programa.exe");
            pausa("Presiona Enter para finalizar ejecución...\n");
            process::exit(1)
        }

        // Usar ruta multiplataforma según el OS
        let path = if cfg!(windows) {
            format!("c:\\xcopy\\tools\\{}.json", tercer_elemento)
        } else {
            format!("./inputs-backup/{}.json", tercer_elemento)
        };

        let data = match fs::read_to_string(&path) {
            Ok(contenido) => contenido,
            Err(e) => {
                eprintln!("❌ No se pudo leer el archivo JSON '{}': {}", path, e);
                pausa("Presiona Enter para finalizar ejecución...\n");
                process::exit(1);
            }
        };

        let init: Inicio = match serde_json::from_str(&data) {
            Ok(obj) => obj,
            Err(e) => {
                eprintln!("❌ Error al parsear el archivo JSON '{}': {}", path, e);
                pausa("Presiona Enter para finalizar ejecución...\n");
                process::exit(1);
            }
        };

        // Leer archivo de rutas (config)
        let config: Config = {
            let data = fs::read_to_string(init.archivo_de_rutas.clone())
                .expect("No se pudo leer el archivo de rutas");
            serde_json::from_str(&data).expect("Error al parsear el archivo de rutas")
        };

        // Leer archivo backup_excluir_carpetas.txt
        let contenido_excluir = fs::read_to_string(init.archivo_excluir_carpetas.clone())
            .expect(&format!(
                "Error al leer el archivo {}, ver el archivo inicio.json",
                init.archivo_excluir_carpetas.clone()
            ));

        (init, config, contenido_excluir)
    }

}

pub mod utils {
    use std::io::{self, Write};
    use std::process::Command;

    pub fn pausa(mensaje: &str) {
        let mut input = String::new();
        print!("{}\n", mensaje);
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut input);
    }

    pub fn hibernar_windows() {
        let _ = Command::new("shutdown")
            .args(&["/h"])
            .output()
            .expect("Error al hibernar");
        println!("El sistema se hibernará.");
    }

    pub fn apagar_windows() {
        let _ = Command::new("shutdown")
            .args(&["/s", "/t", "0"])
            .output()
            .expect("Error al apagar");
        println!("El sistema se apagará.");
    }

}