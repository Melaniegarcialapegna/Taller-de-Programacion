cargo build --release
cd target/release/
./room-rtc ../../archivos_test/config/peer1.conf &
./room-rtc ../../archivos_test/config/peer2.conf &
