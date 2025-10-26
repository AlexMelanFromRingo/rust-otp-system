#!/bin/bash
set -e

echo "=== Сборка системы одноразовых паролей ==="
echo ""

# Установка необходимых целей
echo "Установка toolchains..."
rustup target add i686-pc-windows-gnu 2>/dev/null || echo "i686-pc-windows-gnu уже установлен"
rustup target add x86_64-pc-windows-gnu 2>/dev/null || echo "x86_64-pc-windows-gnu уже установлен"
rustup target add x86_64-unknown-linux-gnu 2>/dev/null || echo "x86_64-unknown-linux-gnu уже установлен"

echo ""
echo "=== Сборка клиента ==="
cd client

targets=(
  i686-pc-windows-gnu
  x86_64-pc-windows-gnu
  x86_64-unknown-linux-gnu
)

for target in "${targets[@]}"; do
  echo "Сборка клиента для $target..."
  cargo build --target "$target" --release
  echo "✓ Клиент для $target готов"
  echo ""
done

cd ..

echo "=== Сборка сервера ==="
cd server

for target in "${targets[@]}"; do
  echo "Сборка сервера для $target..."
  cargo build --target "$target" --release
  echo "✓ Сервер для $target готов"
  echo ""
done

cd ..

echo ""
echo "=== Сборка завершена! ==="
echo ""
echo "Собранные файлы находятся в:"
echo "  • client/target/<target>/release/otp_client[.exe]"
echo "  • server/target/<target>/release/otp_server[.exe]"
echo ""
echo "Доступные цели:"
echo "  • i686-pc-windows-gnu       - Windows 32-bit"
echo "  • x86_64-pc-windows-gnu     - Windows 64-bit"
echo "  • x86_64-unknown-linux-gnu  - Linux 64-bit"
