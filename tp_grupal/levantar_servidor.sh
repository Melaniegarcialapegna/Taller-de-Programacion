#!/bin/bash

#Se da permisos
chmod +x levantar_servidor.sh

#Se compila servidor
cargo build

#Se inicia servidor
cargo run --bin servidor archivos_test/config/servidor.conf &

echo "El servidor de inicio correctamente"
