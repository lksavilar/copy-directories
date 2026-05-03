use chrono::Local;
use std::{
    fs::OpenOptions,
    io::Write, fs::File, io::Result as IoResult
};

use crate::tools;
use crate::platform::{detect_platform, Platform};
use crate::commands::CommandBuilder;

pub fn run_backup() -> IoResult<()> {
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
        Platform::Windows => format!("C:/xcopy/{}-{}{}-{}_{}_resumen_backup_accer_rosita.txt",fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual),
        _ => {
            let dir = logs_dir_linux();
            format!("{}/{}-{}{}-{}_{}_resumen_backup.txt",dir,fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual)
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
        "Resumen del BACKUP: '{}' ",
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

    // Dar formato al nombre del archivo log_backup de salida
    let archivo_salida = match platform {
        Platform::Windows => format!("C:/xcopy/logs/{}-{}{}-{}_{}_log_backup_accer_rosita.txt",fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual),
        _ => {
            let dir = logs_dir_linux();
            format!("{}/{}-{}{}-{}_{}_log_backup.txt",dir,fechaok.get_anio(),fechaok.get_mes_numero(),fechaok.get_mes_nombre3letras(),fechaok.get_dia_numero(),hora_actual)
        }
    };

    for (idx, ruta) in config.rutas_origen_destino.iter().enumerate() {
        let (origen, destino) = ruta;
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
                "* Se inicio el BACKUP: '{}' ",fechaok.get_fecha_formateada()).unwrap();
            writeln!(
                file,
                "***************************************************************"
            )
            .unwrap();
            println!("----------------------------------------------------------------------------");
            println!("SE HA INICIADO EL BACKUP: '{}'.    FAVOR ESPERAR ...",fechaok.get_fecha_formateada());
            println!("----------------------------------------------------------------------------\n");
        }

        // Construir ruta de destino según la plataforma
        let destino_con_ruta = config.build_destination_path(destino, &platform);

        fechaok = tools::structs::fecha();
        println!("###########################################################################################");
        writeln!(file,"###########################################################################################").unwrap();
        println!("Iniciado: {} ", fechaok.get_fecha_formateada());
        println!("Ruta  origen: \"{}{}\"", origen, platform.path_separator());
        println!("Ruta destino: \"{}\"", destino_con_ruta);
        writeln!(file, "Iniciado: {} ", fechaok.get_fecha_formateada()).unwrap();
        writeln!(file, "Ruta origen: \"{}{}\"", origen, platform.path_separator()).unwrap();
        writeln!(file, "Ruta destino: \"{}\"", destino_con_ruta).unwrap();

        // Construir comando multiplataforma
        let mut comando = command_builder.build_backup_command(origen, &destino_con_ruta, &carpetas_excluir);

        // Ejecutar comando de backup multiplataforma
        let output = comando.output().expect(&format!("Error al ejecutar comando de backup en {:?}", platform));

        // Convierte la salida del comando a texto
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("\nArchivos copiados: \n{}", stdout);
        writeln!(file, "\nArchivos copiados: \n{:?} {}: ", output.status.code(),stdout).unwrap(); 

        // Procesar salida según la plataforma
        command_builder.process_output(&stdout, &mut file_rs)?;

        print!("\n\n");

        // Verificar códigos de salida según la plataforma
        if command_builder.is_success_code(output.status.code()) {
            writeln!(file,"----------------------------------------------------------------------------------").unwrap();
            writeln!(
                file,
                "Copia de seguridad de '{}' completada exitosamente.",
                origen
            )
            .unwrap();
            writeln!(file,"----------------------------------------------------------------------------------\n").unwrap();
        } else {
            writeln!(
                file,
                "Error en la copia de seguridad de '{}': {:?}",
                origen, output
            )
            .unwrap();
            // Imprimir el error detallado
            let stderr = String::from_utf8_lossy(&output.stderr);
            writeln!(file, "Detalles del error: {}", stderr).unwrap();
        }

        drop(file);
    }

    fechaok = tools::structs::fecha();

    println!("\nFinal de la copia (BACKUP), {}", fechaok.get_fecha_formateada());
    writeln!(file_rs, "\nFinal de la copia (BACKUP), {}", fechaok.get_fecha_formateada()).unwrap();
    drop(file_rs);

    if platform.is_windows() {
        let apagar = init.apagar_equipo.to_lowercase();
        if apagar == "y" || apagar == "s" {
            tools::utils::hibernar_windows();
        } else if apagar == "off" {
            tools::utils::apagar_windows();
        } else {
            println!("Para salir y finalizar el proceso de backup...");
            tools::utils::pausa("Presiona Enter \n");
        }
    } else {
        println!("Para salir y finalizar el proceso de backup...");
        tools::utils::pausa("Presiona Enter \n");
    }

    Ok(())
}

fn logs_dir_linux() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = format!("{}/logs/backup", home);
    let _ = std::fs::create_dir_all(&dir);
    dir
}