use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    match rompecabezas_de_las_sombras::ejecutar_programa(&mut stdin, &mut stdout, &mut stderr) {
        Ok(()) => {}
        Err(error) => {
            let _ = writeln!(stderr, "{}", error);
        }
    }
}
