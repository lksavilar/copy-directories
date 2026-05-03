use std::process::Command;
use std::io::{Write, Result as IoResult};
use regex::Regex;

use crate::platform::Platform;

pub struct CommandBuilder {
    platform: Platform,
}

impl CommandBuilder {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }

    pub fn build_backup_command(
        &self,
        origen: &str,
        destino: &str,
        carpetas_excluir: &[&str],
    ) -> Command {
        match self.platform {
            Platform::Windows => self.build_robocopy_command(origen, destino, carpetas_excluir),
            Platform::Linux | Platform::MacOS => self.build_rsync_command(origen, destino, carpetas_excluir),
            Platform::Unknown => self.build_rsync_command(origen, destino, carpetas_excluir),
        }
    }

    fn build_robocopy_command(
        &self,
        origen: &str,
        destino: &str,
        carpetas_excluir: &[&str],
    ) -> Command {
        let mut comando = Command::new("robocopy");
        
        comando
            .arg(format!(r"{}", origen))
            .arg(format!(r"{}", destino))
            .arg("/E")          // Copiar todos los subdirectorios
            .arg("/J")          // E/S no almacenada en búfer
            .arg("/COPY:DAT")   // Copiar Datos, Atributos, Tiempo
            .arg("/DCOPY:DAT")  // Copiar directorios: Datos, Atributos, Tiempo
            .arg("/R:3")        // Reintentar 3 veces
            .arg("/W:5")        // Esperar 5 segundos entre reintentos
            .arg("/MT:16")      // 16 hilos paralelos
            .arg("/FP")         // Ruta completa en resultado
            .arg("/NJH")        // Sin encabezado de trabajo
            .arg("/UNICODE");   // Mostrar como UNICODE

        // Agregar carpetas a excluir
        if !carpetas_excluir.is_empty() {
            comando.arg("/XD");
            for carpeta in carpetas_excluir {
                comando.arg(carpeta);
            }
        }

        comando
    }

    fn build_rsync_command(
        &self,
        origen: &str,
        destino: &str,
        carpetas_excluir: &[&str],
    ) -> Command {
        let mut comando = Command::new("rsync");
        
        comando
            .arg("-avz")              // Archive mode + verbose + compress
            .arg("--itemize-changes") // Prefijos >, c, * necesarios para parsear la salida
            .arg("--progress")        // Mostrar progreso
            .arg("--human-readable")  // Tamaños legibles
            .arg("--stats");          // Mostrar estadísticas

        // Agregar carpetas a excluir
        for carpeta in carpetas_excluir {
            comando.arg("--exclude").arg(carpeta);
        }

        // Crear directorio de destino antes de la copia
        if let Some(parent) = std::path::Path::new(destino).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Asegurar que origen termine con / para rsync
        let origen_normalizado = if origen.ends_with('/') {
            origen.to_string()
        } else {
            format!("{}/", origen)
        };

        comando.arg(&origen_normalizado).arg(destino);

        comando
    }

    pub fn process_output(&self, stdout: &str, file_rs: &mut dyn Write) -> IoResult<()> {
        match self.platform {
            Platform::Windows => self.process_robocopy_output(stdout, file_rs),
            Platform::Linux | Platform::MacOS => self.process_rsync_output(stdout, file_rs),
            Platform::Unknown => self.process_rsync_output(stdout, file_rs),
        }
    }

    // Expresión regular para capturar 'Nuevo arch', 'reciente' o '*Archivo EXTRA' y la ruta que comienza con 'C:\', 'D:\' o 'G:\'
    fn process_robocopy_output(&self, stdout: &str, file_rs: &mut dyn Write) -> IoResult<()> {
        let re = Regex::new(r"(\b\d{1,3}%\b\s*)?\s*((Nuevo arch)|(M�s reciente)|(antiguo)|(\*Directorio EXTRA)|(\*Archivo EXTRA))\s+([-+]?\d*\.?\d+\s*m?)?\s+([CDG]:\\[^\s]+(?:\s+[^\s]+)*)").unwrap();

        if stdout.contains("Nuevo arch")
            || stdout.contains("reciente")
            || stdout.contains("antiguo")
            || stdout.contains("Archivo EXTRA")
            || stdout.contains("Directorio EXTRA")
        {
            for line in stdout.lines() {
                if line.contains("Nuevo arch")
                    || line.contains("s reciente")
                    || line.contains("s antiguo")
                    || line.contains("o EXTRA")
                {
                    let mut linex: String = line.to_string();

                    // Limpiar caracteres de control ASCII
                    let rex = Regex::new(r"[\x00-\x1F\x7F\t]+").unwrap();
                    linex = rex.replace_all(&linex, " ").to_string();

                    // Limpiar patrones de porcentaje
                    let rex = Regex::new(r"\b(?:100|\b(?:[1-9]?\d))%").unwrap();
                    linex = rex.replace_all(&linex, " ").to_string();

                    if let Some(captures) = re.captures(&linex) {
                        let percentage = captures.get(1).map_or("", |m| m.as_str());
                        let mut keyword = captures.get(2).map_or("", |m| m.as_str());
                        let path = captures.get(9).map_or("", |m| m.as_str());
                        
                        keyword = match keyword {
                            "Nuevo arch" => "Nuevo Archivo ---------------->",
                            "M�s reciente" => "Modificado (Más reciente) ---->",
                            "antiguo" => "Origen Desactualizado -------->",
                            "*Directorio EXTRA" => "Directorio Borrado en Origen ->",
                            "*Archivo EXTRA" => "Archivo Borrado en Origen ---->",
                            _ => keyword
                        };

                        if percentage.is_empty() {
                            println!("{}\t{}", keyword, path);
                            writeln!(file_rs, "{}\t{}", keyword, path)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn process_rsync_output(&self, stdout: &str, file_rs: &mut dyn Write) -> IoResult<()> {
        // Procesar salida de rsync
        for line in stdout.lines() {
            // rsync muestra archivos transferidos línea por línea
            if !line.is_empty() && !line.starts_with("sending") && !line.starts_with("sent") && !line.starts_with("total") {
                // Detectar tipo de operación y limpiar línea
                if line.starts_with('>') {
                    let clean_line = line.trim_start_matches('>').trim();
                    if !clean_line.is_empty() {
                        println!("Archivo Transferido ---------->\t{}", clean_line);
                        writeln!(file_rs, "Archivo Transferido ---------->\t{}", clean_line)?;
                    }
                } else if line.starts_with('c') {
                    let clean_line = line.trim_start_matches('c').trim();
                    if !clean_line.is_empty() {
                        println!("Archivo Creado -------------->\t{}", clean_line);
                        writeln!(file_rs, "Archivo Creado -------------->\t{}", clean_line)?;
                    }
                } else if line.starts_with('*') {
                    let clean_line = line.trim_start_matches('*').trim();
                    if !clean_line.is_empty() {
                        println!("Archivo Eliminado ----------->\t{}", clean_line);
                        writeln!(file_rs, "Archivo Eliminado ----------->\t{}", clean_line)?;
                    }
                } else if line.contains("deleting") {
                    let cleaned = line.replace("deleting ", "");
                    let clean_line = cleaned.trim();
                    if !clean_line.is_empty() {
                        println!("Archivo Borrado en Destino -->\t{}", clean_line);
                        writeln!(file_rs, "Archivo Borrado en Destino -->\t{}", clean_line)?;
                    }
                } else {
                    let clean_line = line.trim();
                    if !clean_line.is_empty() {
                        println!("Archivo Procesado ----------->\t{}", clean_line);
                        writeln!(file_rs, "Archivo Procesado ----------->\t{}", clean_line)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn is_success_code(&self, code: Option<i32>) -> bool {
        match self.platform {
            Platform::Windows => {
                // Códigos 0-7: operación exitosa (con o sin diferencias). 8+ son errores reales.
                matches!(code, Some(0) | Some(1) | Some(2) | Some(3) | Some(4) | Some(5) | Some(6) | Some(7))
            }
            Platform::Linux | Platform::MacOS => {
                // Código de éxito de rsync
                code == Some(0)
            }
            Platform::Unknown => code == Some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Códigos de salida RoboCopy ---

    #[test]
    fn test_robocopy_codigos_0_a_7_son_exito() {
        let builder = CommandBuilder::new(Platform::Windows);
        for code in 0i32..=7 {
            assert!(
                builder.is_success_code(Some(code)),
                "RoboCopy: código {} debería ser éxito", code
            );
        }
    }

    #[test]
    fn test_robocopy_codigos_8_en_adelante_son_error() {
        let builder = CommandBuilder::new(Platform::Windows);
        for code in [8i32, 9, 10, 11, 16] {
            assert!(
                !builder.is_success_code(Some(code)),
                "RoboCopy: código {} debería ser error", code
            );
        }
    }

    #[test]
    fn test_robocopy_none_es_error() {
        let builder = CommandBuilder::new(Platform::Windows);
        assert!(!builder.is_success_code(None));
    }

    // --- Códigos de salida rsync ---

    #[test]
    fn test_rsync_codigo_0_es_exito() {
        let builder = CommandBuilder::new(Platform::Linux);
        assert!(builder.is_success_code(Some(0)));
    }

    #[test]
    fn test_rsync_codigos_distintos_de_0_son_error() {
        let builder = CommandBuilder::new(Platform::Linux);
        for code in [1i32, 2, 3, 11, 23, 24] {
            assert!(
                !builder.is_success_code(Some(code)),
                "rsync: código {} debería ser error", code
            );
        }
    }

    #[test]
    fn test_rsync_none_es_error() {
        let builder = CommandBuilder::new(Platform::Linux);
        assert!(!builder.is_success_code(None));
    }

    // --- Parseo de salida rsync ---

    #[test]
    fn test_rsync_detecta_archivo_transferido() {
        let builder = CommandBuilder::new(Platform::Linux);
        let mut buf: Vec<u8> = Vec::new();
        builder.process_rsync_output(">f+++++++++ docs/readme.txt\n", &mut buf).unwrap();
        let resultado = String::from_utf8(buf).unwrap();
        assert!(resultado.contains("Archivo Transferido"), "Debe detectar '>'");
        assert!(resultado.contains("docs/readme.txt"));
    }

    #[test]
    fn test_rsync_detecta_archivo_creado() {
        let builder = CommandBuilder::new(Platform::Linux);
        let mut buf: Vec<u8> = Vec::new();
        builder.process_rsync_output("cf+++++++++ src/main.rs\n", &mut buf).unwrap();
        let resultado = String::from_utf8(buf).unwrap();
        assert!(resultado.contains("Archivo Creado"), "Debe detectar 'c'");
        assert!(resultado.contains("src/main.rs"));
    }

    #[test]
    fn test_rsync_detecta_archivo_eliminado_con_asterisco() {
        let builder = CommandBuilder::new(Platform::Linux);
        let mut buf: Vec<u8> = Vec::new();
        builder.process_rsync_output("*deleting viejo.txt\n", &mut buf).unwrap();
        let resultado = String::from_utf8(buf).unwrap();
        assert!(resultado.contains("Archivo Eliminado"), "Debe detectar '*'");
    }

    #[test]
    fn test_rsync_detecta_borrado_con_deleting() {
        let builder = CommandBuilder::new(Platform::Linux);
        let mut buf: Vec<u8> = Vec::new();
        builder.process_rsync_output("deleting carpeta/archivo.txt\n", &mut buf).unwrap();
        let resultado = String::from_utf8(buf).unwrap();
        assert!(resultado.contains("Archivo Borrado en Destino"), "Debe detectar 'deleting'");
        assert!(resultado.contains("carpeta/archivo.txt"));
    }

    #[test]
    fn test_rsync_ignora_lineas_vacias() {
        let builder = CommandBuilder::new(Platform::Linux);
        let mut buf: Vec<u8> = Vec::new();
        builder.process_rsync_output("\n\n\n", &mut buf).unwrap();
        assert!(buf.is_empty(), "Líneas vacías no deben generar salida");
    }

    #[test]
    fn test_rsync_ignora_lineas_de_estadisticas() {
        let builder = CommandBuilder::new(Platform::Linux);
        let mut buf: Vec<u8> = Vec::new();
        let output = "sending incremental file list\nsent 1234 bytes\ntotal size is 5678\n";
        builder.process_rsync_output(output, &mut buf).unwrap();
        let resultado = String::from_utf8(buf).unwrap();
        assert!(!resultado.contains("sending"));
        assert!(!resultado.contains("sent"));
        assert!(!resultado.contains("total"));
    }

    // --- Construcción del comando rsync ---

    #[test]
    fn test_rsync_comando_incluye_itemize_changes() {
        let builder = CommandBuilder::new(Platform::Linux);
        let cmd = builder.build_backup_command("/origen", "/destino", &[]);
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(args.contains(&"--itemize-changes".to_string()));
    }

    #[test]
    fn test_rsync_origen_termina_con_barra() {
        let builder = CommandBuilder::new(Platform::Linux);
        let cmd = builder.build_backup_command("/origen/sin/barra", "/destino", &[]);
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(
            args.iter().any(|a| a == "/origen/sin/barra/"),
            "El origen debe terminar con / para rsync"
        );
    }

    #[test]
    fn test_rsync_origen_ya_con_barra_no_duplica() {
        let builder = CommandBuilder::new(Platform::Linux);
        let cmd = builder.build_backup_command("/origen/con/barra/", "/destino", &[]);
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(
            !args.iter().any(|a| a == "/origen/con/barra//"),
            "No debe duplicar la barra final"
        );
    }

    #[test]
    fn test_rsync_exclusiones_se_agregan_correctamente() {
        let builder = CommandBuilder::new(Platform::Linux);
        let exclusiones = ["node_modules", ".git", "target"];
        let cmd = builder.build_backup_command("/origen", "/destino", &exclusiones);
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(args.contains(&"--exclude".to_string()));
        for excluida in &exclusiones {
            assert!(args.contains(&excluida.to_string()), "Falta exclusión: {}", excluida);
        }
    }

    // --- Programa correcto según plataforma ---

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_programa_en_linux_es_rsync() {
        let builder = CommandBuilder::new(Platform::Linux);
        let cmd = builder.build_backup_command("/origen", "/destino", &[]);
        assert_eq!(cmd.get_program(), "rsync");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_programa_en_windows_es_robocopy() {
        let builder = CommandBuilder::new(Platform::Windows);
        let cmd = builder.build_backup_command("C:\\origen", "D:\\destino", &[]);
        assert_eq!(cmd.get_program(), "robocopy");
    }
}