#!/bin/bash

echo "Compilando..."
cargo build

echo "Iniciando clientes..."
cargo run --bin room-rtc archivos_test/config/peer1.conf &
cargo run --bin room-rtc archivos_test/config/peer2.conf &

echo "Se iniciaron los dos clientes"
