use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        match env::consts::OS {
            "windows" => Platform::Windows,
            "linux" => Platform::Linux,
            "macos" => Platform::MacOS,
            _ => Platform::Unknown,
        }
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::Windows)
    }

    pub fn path_separator(&self) -> &str {
        match self {
            Platform::Windows => "\\",
            Platform::Linux | Platform::MacOS => "/",
            Platform::Unknown => "/",
        }
    }

    pub fn normalize_path(&self, path: &str) -> String {
        match self {
            Platform::Windows => path.replace("/", "\\"),
            Platform::Linux | Platform::MacOS => path.replace("\\", "/"),
            Platform::Unknown => path.replace("\\", "/"),
        }
    }

}

pub fn detect_platform() -> Platform {
    Platform::current()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Separadores de ruta ---

    #[test]
    fn test_separador_windows() {
        assert_eq!(Platform::Windows.path_separator(), "\\");
    }

    #[test]
    fn test_separador_linux() {
        assert_eq!(Platform::Linux.path_separator(), "/");
    }

    #[test]
    fn test_separador_macos() {
        assert_eq!(Platform::MacOS.path_separator(), "/");
    }

    #[test]
    fn test_separador_unknown_usa_slash() {
        assert_eq!(Platform::Unknown.path_separator(), "/");
    }

    // --- Normalización de rutas ---

    #[test]
    fn test_normalize_linux_convierte_backslashes() {
        let result = Platform::Linux.normalize_path("C:\\Users\\test\\archivo.txt");
        assert_eq!(result, "C:/Users/test/archivo.txt");
    }

    #[test]
    fn test_normalize_linux_no_altera_ruta_correcta() {
        let result = Platform::Linux.normalize_path("/home/user/docs");
        assert_eq!(result, "/home/user/docs");
    }

    #[test]
    fn test_normalize_windows_convierte_forward_slashes() {
        let result = Platform::Windows.normalize_path("/home/user/docs");
        assert_eq!(result, "\\home\\user\\docs");
    }

    #[test]
    fn test_normalize_windows_no_altera_ruta_correcta() {
        let result = Platform::Windows.normalize_path("C:\\Users\\test");
        assert_eq!(result, "C:\\Users\\test");
    }

    #[test]
    fn test_normalize_unknown_se_comporta_como_linux() {
        let result = Platform::Unknown.normalize_path("C:\\folder\\file");
        assert_eq!(result, "C:/folder/file");
    }

    // --- is_windows ---

    #[test]
    fn test_is_windows_solo_verdadero_en_windows() {
        assert!(Platform::Windows.is_windows());
        assert!(!Platform::Linux.is_windows());
        assert!(!Platform::MacOS.is_windows());
        assert!(!Platform::Unknown.is_windows());
    }

    // --- detect_platform coincide con el OS real ---

    #[test]
    #[cfg(target_os = "windows")]
    fn test_detect_platform_en_windows() {
        assert_eq!(detect_platform(), Platform::Windows);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_detect_platform_en_linux() {
        assert_eq!(detect_platform(), Platform::Linux);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_detect_platform_en_macos() {
        assert_eq!(detect_platform(), Platform::MacOS);
    }
}