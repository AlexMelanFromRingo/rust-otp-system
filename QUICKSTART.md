# 🚀 Быстрый старт

## Установка зависимостей (один раз)

```bash
# В WSL2 выполните:
sudo apt update
sudo apt install mingw-w64

# Установка Rust (если еще не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Добавление целевых платформ
rustup target add i686-pc-windows-gnu
rustup target add x86_64-pc-windows-gnu  
rustup target add x86_64-unknown-linux-gnu
```

## Сборка

```bash
cd otp-system
chmod +x build.sh
./build.sh
```

Или для одной платформы:

```bash
# Windows 64-bit
cd client && cargo build --target x86_64-pc-windows-gnu --release
cd ../server && cargo build --target x86_64-pc-windows-gnu --release

# Linux
cd client && cargo build --target x86_64-unknown-linux-gnu --release
cd ../server && cargo build --target x86_64-unknown-linux-gnu --release
```

## Где найти собранные файлы

- **Windows 32-bit**: `*/target/i686-pc-windows-gnu/release/*.exe`
- **Windows 64-bit**: `*/target/x86_64-pc-windows-gnu/release/*.exe`
- **Linux**: `*/target/x86_64-unknown-linux-gnu/release/*`

## Быстрый тест

```bash
# Создайте тестовые файлы
echo "e2d76510bf24" > bs1.txt
echo "06.05.2007 21:24:30" > bs3.txt
echo "Тест_Т.Т. test AAAAe2d76510bf24 06.05.2007 21:24:30" > database.txt

# Запустите программы (Linux)
cd client/target/x86_64-unknown-linux-gnu/release
./otp_client &

cd ../../server/target/x86_64-unknown-linux-gnu/release  
./otp_server &
```

Или просто откройте .exe файлы в Windows!

## Совместимость с оригинальными файлами

Положите новые .exe/.bin файлы рядом со старыми файлами bs1.txt, bs3.txt, database.txt - все будет работать! ✅
