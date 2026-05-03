use chrono::Local;
use std::{
    fs::OpenOptions,
    io::Write, fs::File, io::Result as IoResult
};

use crate::tools;
use crate::platform::{detect_platform, Platform};
use crate::commands::CommandBuilder;

pub fn run_restore(path_filter: Option<String>, force: bool) -> IoResult<()> {
    let platform = detect_platform();
    let command_builder = CommandBuilder::new(platform);
    
    println!("🖥️  Detectado: {:?}", platform);
    
    let mut fechaok = tools::structs::fecha();

    let (init, mut config, contenido_excluir) = tools::config_loader::cargar_configuracion();
    
    // Normalizar rutas según la plataforma
    config.normalize_paths(&platform);

    // Separar las líneas filtrando comentarios y líneas vacías
    let carpetas_excluir: Vec<&str> = contenido_excluir
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .collect();

    let hora_actual = Local::now().format("%Hhor%Mmin").to_string();

    // Adaptación de ruta de salida según plataforma
    let resumen_salida = match platform {
        Platform::Windows => format!("C:/xcopy/{}-{}{}-{}_{}_resumen_restore_accer_rosita.txt",fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual),
        _ => {
            let dir = logs_dir_linux();
            format!("{}/{}-{}{}-{}_{}_resumen_restore.txt",dir,fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual)
        }
    };

    // Usamos OpenOptions para abrir un archivo en modo de escritura
    let mut file_rs = OpenOptions::new()
        .write(true)   // Permitir escritura
        .append(true)  // Permitir añadir contenido al final del archivo
        .create(true)  // Crear el archivo si no existe
        .open(&resumen_salida)?;

    writeln!(
        file_rs,
        "--------------------------------------------------------------------------------------"
    )
    .unwrap();
    writeln!(
        file_rs,
        "Resumen del RESTORE: '{}' ",
        fechaok.get_fecha_formateada()
    )
    .unwrap();
    writeln!(
        file_rs,
        "--------------------------------------------------------------------------------------"
    )
    .unwrap();

    // Verificar directorio/unidad de backup antes de iniciar
    tools::check_drive_backup::check_drive_backup(&init, &config, &mut file_rs)?;

    // Dar formato al nombre del archivo log_restore de salida
    let archivo_salida = match platform {
        Platform::Windows => format!("C:/xcopy/logs/{}-{}{}-{}_{}_log_restore_accer_rosita.txt",fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual),
        _ => {
            let dir = logs_dir_linux();
            format!("{}/{}-{}{}-{}_{}_log_restore.txt",dir,fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual)
        }
    };

    // Confirmación de seguridad si no se usa --force
    if !force {
        println!("⚠️  ADVERTENCIA: Esta operación restaurará archivos desde el backup.");
        println!("Esto puede sobrescribir archivos existentes en las rutas de destino.");
        
        if let Some(ref filter_path) = path_filter {
            println!("Ruta a restaurar: {}", filter_path);
        } else {
            println!("Se restaurarán TODAS las rutas configuradas:");
            for (_, destino) in &config.rutas_origen_destino {
                println!("  - {}", destino);
            }
        }
        
        print!("\n¿Desea continuar? (s/N): ");
        std::io::stdout().flush().unwrap();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        
        if input != "s" && input != "si" && input != "y" && input != "yes" {
            println!("Operación cancelada.");
            return Ok(());
        }
    }

    // Filtrar rutas si se especifica un filtro
    let rutas_a_restaurar: Vec<_> = if let Some(ref filter) = path_filter {
        config.rutas_origen_destino.iter()
            .filter(|(_, destino)| destino.contains(filter))
            .collect()
    } else {
        config.rutas_origen_destino.iter().collect()
    };

    if rutas_a_restaurar.is_empty() {
        println!("No se encontraron rutas que coincidan con el filtro: {:?}", path_filter);
        return Ok(());
    }

    for (idx, ruta) in rutas_a_restaurar.iter().enumerate() {
        let (origen_original, destino) = ruta;
        // Actualizar la estructura con la fecha y hora actuales
        fechaok = tools::structs::fecha();
 
        let mut file: File = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&archivo_salida)
            .expect("No se pudo abrir el archivo de salida");

        if idx == 0 {
            writeln!(
                file,
                "\n\n***************************************************************"
            )
            .unwrap();
            writeln!(
                file,
                "* Se inicio el RESTORE: '{}' ",fechaok.get_fecha_formateada()).unwrap();
            writeln!(
                file,
                "***************************************************************"
            )
            .unwrap();
            println!("----------------------------------------------------------------------------");
            println!("SE HA INICIADO EL RESTORE: '{}'.    FAVOR ESPERAR ...",fechaok.get_fecha_formateada());
            println!("----------------------------------------------------------------------------\n");
        }

        // Para restore: origen está en el backup, destino es la ubicación original
        // Construir la ruta en el backup (lo que era destino en backup, ahora es origen)
        let origen_backup = config.build_destination_path(destino, &platform);
        let destino_restore = origen_original; // Restaurar a la ubicación original

        fechaok = tools::structs::fecha();
        println!("###########################################################################################");
        writeln!(file,"###########################################################################################").unwrap();
        println!("Iniciado: {} ", fechaok.get_fecha_formateada());
        println!("Restaurando desde: \"{}\"", origen_backup);
        println!("Restaurando hacia: \"{}{}\"", destino_restore, platform.path_separator());
        writeln!(file, "Iniciado: {} ", fechaok.get_fecha_formateada()).unwrap();
        writeln!(file, "Restaurando desde: \"{}\"", origen_backup).unwrap();
        writeln!(file, "Restaurando hacia: \"{}{}\"", destino_restore, platform.path_separator()).unwrap();

        // Construir comando multiplataforma para restore
        let mut comando = command_builder.build_backup_command(&origen_backup, destino_restore, &carpetas_excluir);

        // Ejecutar comando de restore multiplataforma
        let output = comando.output().expect(&format!("Error al ejecutar comando de restore en {:?}", platform));

        // Convierte la salida del comando a texto
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("\nArchivos restaurados: \n{}", stdout);
        writeln!(file, "\nArchivos restaurados: \n{:?} {}: ", output.status.code(),stdout).unwrap(); 

        // Procesar salida según la plataforma
        command_builder.process_output(&stdout, &mut file_rs)?;

        print!("\n\n");

        // Verificar códigos de salida según la plataforma
        if command_builder.is_success_code(output.status.code()) {
            writeln!(file,"----------------------------------------------------------------------------------").unwrap();
            writeln!(
                file,
                "Restauración de '{}' completada exitosamente.",
                destino_restore
            )
            .unwrap();
            writeln!(file,"----------------------------------------------------------------------------------\n").unwrap();
        } else {
            writeln!(
                file,
                "Error en la restauración de '{}': {:?}",
                destino_restore, output
            )
            .unwrap();
            // Imprimir el error detallado
            let stderr = String::from_utf8_lossy(&output.stderr);
            writeln!(file, "Detalles del error: {}", stderr).unwrap();
        }

        drop(file);
    }

    fechaok = tools::structs::fecha();
    println!("\nRestauración completada.");
    println!("Final del RESTORE: {}", fechaok.get_fecha_formateada());
    writeln!(file_rs, "\nFinal del RESTORE: {}", fechaok.get_fecha_formateada()).unwrap();
    drop(file_rs);

    println!("Para salir y finalizar el proceso de restore...");
    tools::utils::pausa("Presiona Enter \n");

    Ok(())
}

fn logs_dir_linux() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = format!("{}/logs/backup", home);
    let _ = std::fs::create_dir_all(&dir);
    dir
}
