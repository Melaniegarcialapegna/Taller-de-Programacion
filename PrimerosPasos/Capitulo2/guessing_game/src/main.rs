use std::{cmp::Ordering, io}; //io input/output libreria
use rand::Rng;

fn main() {
    println!("Adivina el numero!");

    let numero_secreto = rand::thread_rng().gen_range(1..=100);

    println!("El numero secreto es {numero_secreto}");

    loop{

    println!("Por favor ingresa tu prediccion!");

    let mut prediccion = String::new();

    io::stdin()
        .read_line(&mut prediccion)
        .expect("Error al leer la linea");

    //CUANDO ERROR => MANEJANDO
    //Para no crashear el programa cuando se ingresa algo que no es un numero
    let prediccion: u32 = match prediccion.trim().parse(){
        Ok(num)=> num,
        Err(_) => {
            println!("Por favor ingrese un numero");            
            continue;
        } //_ es un catch all
    };
    
    //CUANDO ERROR => CRASHEO
    //Con esto crashearia pq estoy utilizando un .exxpect()
    //let prediccion: u32 = prediccion.trim().parse().expect("Por favor escribi un numero");

    println!("Tu prediccion : {prediccion}");

    match prediccion.cmp(&numero_secreto){ //compara el numero secreto con el de prediccion
        Ordering::Less=> println!("Muy chiquito"),
        Ordering::Greater=> println!("Muy grande"),
        Ordering::Equal=> {
            println!("Adivinaste!!¡¡");
            break;
        }
    }
    //.cmp devuelve una variante de Ordering(es un enum)
    //Si el enum es igual a Less hago tal cosa, si es igual a Greater otra y asi ...
    }
}
