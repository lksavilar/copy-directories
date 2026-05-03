use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Inicio {
    pub archivo_de_rutas: String,
    pub archivo_excluir_carpetas: String,
    pub apagar_equipo: String, // puede ser "y", "off", "n", etc.
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub rutas_origen_destino: Vec<(String, String)>,
    pub destino_final: String,
    pub name_disk: String,
}

impl Config {
    pub fn normalize_paths(&mut self, platform: &crate::platform::Platform) {
        // Normalizar rutas según la plataforma
        for (origen, destino) in &mut self.rutas_origen_destino {
            *origen = platform.normalize_path(origen);
            *destino = platform.normalize_path(destino);
        }
        self.destino_final = platform.normalize_path(&self.destino_final);
    }

    pub fn build_destination_path(&self, destino: &str, platform: &crate::platform::Platform) -> String {
        let separator = platform.path_separator();
        // Solo remover los ":" pero mantener los separadores de ruta
        let destino_limpio = destino.replace(":", "");
        // Normalizar separadores según la plataforma
        let destino_normalizado = platform.normalize_path(&destino_limpio);
        
        // Asegurar que destino_final termine con separador
        let destino_final_normalizado = if self.destino_final.ends_with('\\') || self.destino_final.ends_with('/') {
            self.destino_final.trim_end_matches(['\\', '/'])
        } else {
            &self.destino_final
        };
        
        format!("{}{}{}", destino_final_normalizado, separator, destino_normalizado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;

    fn config_base(destino_final: &str) -> Config {
        Config {
            rutas_origen_destino: vec![],
            destino_final: destino_final.to_string(),
            name_disk: "BACKUP".to_string(),
        }
    }

    // --- build_destination_path ---

    #[test]
    fn test_ruta_destino_linux_basica() {
        let config = config_base("/mnt/g");
        let result = config.build_destination_path("wsl/rust_projects", &Platform::Linux);
        assert_eq!(result, "/mnt/g/wsl/rust_projects");
    }

    #[test]
    fn test_ruta_destino_con_barra_final_no_duplica_separador() {
        let config = config_base("/mnt/g/");
        let result = config.build_destination_path("wsl/docs", &Platform::Linux);
        assert!(!result.contains("//"), "No debe haber doble barra");
        assert_eq!(result, "/mnt/g/wsl/docs");
    }

    #[test]
    fn test_ruta_destino_con_doble_barra_final_se_normaliza() {
        let config = config_base("/mnt/g//");
        let result = config.build_destination_path("wsl/docs", &Platform::Linux);
        assert!(!result.contains("//"));
    }

    #[test]
    fn test_ruta_destino_elimina_dos_puntos() {
        let config = config_base("/mnt/g");
        let result = config.build_destination_path("C:\\Users\\test", &Platform::Linux);
        assert!(!result.contains(':'), "El resultado no debe contener ':'");
    }

    #[test]
    fn test_ruta_destino_windows_usa_backslash() {
        let config = config_base("G:\\backup");
        let result = config.build_destination_path("Users\\test", &Platform::Windows);
        assert!(result.contains('\\'));
        assert_eq!(result, "G:\\backup\\Users\\test");
    }

    #[test]
    fn test_ruta_destino_windows_con_barra_final_no_duplica() {
        let config = config_base("G:\\backup\\");
        let result = config.build_destination_path("Users\\docs", &Platform::Windows);
        assert!(!result.contains("\\\\"), "No debe haber doble backslash de separador");
    }

    // --- normalize_paths ---

    #[test]
    fn test_normalize_linux_convierte_backslashes_en_rutas() {
        let mut config = Config {
            rutas_origen_destino: vec![
                ("C:\\Users\\test".to_string(), "wsl\\test".to_string()),
            ],
            destino_final: "\\mnt\\g".to_string(),
            name_disk: "BACKUP".to_string(),
        };
        config.normalize_paths(&Platform::Linux);
        assert_eq!(config.rutas_origen_destino[0].0, "C:/Users/test");
        assert_eq!(config.rutas_origen_destino[0].1, "wsl/test");
        assert_eq!(config.destino_final, "/mnt/g");
    }

    #[test]
    fn test_normalize_windows_convierte_forward_slashes_en_rutas() {
        let mut config = Config {
            rutas_origen_destino: vec![
                ("/home/user/docs".to_string(), "home/user/docs".to_string()),
            ],
            destino_final: "/mnt/backup".to_string(),
            name_disk: "BACKUP".to_string(),
        };
        config.normalize_paths(&Platform::Windows);
        assert_eq!(config.rutas_origen_destino[0].0, "\\home\\user\\docs");
        assert_eq!(config.rutas_origen_destino[0].1, "home\\user\\docs");
        assert_eq!(config.destino_final, "\\mnt\\backup");
    }

    #[test]
    fn test_normalize_multiples_rutas() {
        let mut config = Config {
            rutas_origen_destino: vec![
                ("/home/user/a".to_string(), "wsl/a".to_string()),
                ("/home/user/b".to_string(), "wsl/b".to_string()),
                ("/home/user/c".to_string(), "wsl/c".to_string()),
            ],
            destino_final: "/mnt/g".to_string(),
            name_disk: "BACKUP".to_string(),
        };
        config.normalize_paths(&Platform::Linux);
        // En Linux las rutas ya están correctas, no deben cambiar
        assert_eq!(config.rutas_origen_destino[0].0, "/home/user/a");
        assert_eq!(config.rutas_origen_destino[2].1, "wsl/c");
    }

    // --- Parseo JSON de Config e Inicio ---

    #[test]
    fn test_parseo_json_inicio_valido() {
        let json = r#"{
            "archivo_de_rutas": "./inputs-backup/rutas.json",
            "archivo_excluir_carpetas": "./inputs-backup/excluir.txt",
            "apagar_equipo": "n"
        }"#;
        let inicio: Inicio = serde_json::from_str(json).expect("Debe parsear correctamente");
        assert_eq!(inicio.apagar_equipo, "n");
        assert_eq!(inicio.archivo_de_rutas, "./inputs-backup/rutas.json");
    }

    #[test]
    fn test_parseo_json_inicio_apagar_y() {
        let json = r#"{
            "archivo_de_rutas": "./rutas.json",
            "archivo_excluir_carpetas": "./excluir.txt",
            "apagar_equipo": "y"
        }"#;
        let inicio: Inicio = serde_json::from_str(json).unwrap();
        assert_eq!(inicio.apagar_equipo, "y");
    }

    #[test]
    fn test_parseo_json_config_valido() {
        let json = r#"{
            "rutas_origen_destino": [
                ["/home/user/docs", "wsl/docs"],
                ["/home/user/projects", "wsl/projects"]
            ],
            "destino_final": "/mnt/g",
            "name_disk": "BACKUP-LKS"
        }"#;
        let config: Config = serde_json::from_str(json).expect("Debe parsear correctamente");
        assert_eq!(config.rutas_origen_destino.len(), 2);
        assert_eq!(config.rutas_origen_destino[0].0, "/home/user/docs");
        assert_eq!(config.rutas_origen_destino[1].1, "wsl/projects");
        assert_eq!(config.destino_final, "/mnt/g");
        assert_eq!(config.name_disk, "BACKUP-LKS");
    }

    #[test]
    fn test_parseo_json_config_sin_rutas() {
        let json = r#"{
            "rutas_origen_destino": [],
            "destino_final": "/mnt/g",
            "name_disk": "BACKUP"
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.rutas_origen_destino.is_empty());
    }

    #[test]
    fn test_parseo_json_config_invalido_falla() {
        let json = r#"{ "campo_inexistente": true }"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err(), "JSON inválido debe retornar error");
    }
}

