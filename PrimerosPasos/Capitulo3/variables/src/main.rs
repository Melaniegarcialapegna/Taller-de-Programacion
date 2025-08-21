fn main() {
    //Ejemplo 1
    let mut x = 5;
    println!("El valor de de x es: {x}");
    x=6;
    println!("El valor de de x es: {x}");

    //Ejemplo2
    let y = 4;
    let y = y + 1; //la pisa con el let

    {
        let y = y*2;
        println!("El valor de ydentro del scope es: {y}")
    }

    print!("El valor de y fuera del scope es: {y}")

}
