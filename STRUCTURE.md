# Структура проекта

```
otp-system/
├── README.md           # Полная документация
├── QUICKSTART.md       # Быстрый старт
├── EXAMPLES.md         # Примеры конфигурации
├── build.sh            # Скрипт сборки для всех платформ
├── Cargo.toml          # Workspace конфигурация
├── .gitignore          # Git ignore файл
│
├── client/             # Генератор одноразовых паролей
│   ├── Cargo.toml      # Зависимости клиента
│   └── src/
│       └── main.rs     # Исходный код клиента (GUI + логика)
│
└── server/             # Сервер аутентификации
    ├── Cargo.toml      # Зависимости сервера
    └── src/
        └── main.rs     # Исходный код сервера (GUI + логика)
```

## После сборки

```
client/target/
├── i686-pc-windows-gnu/
│   └── release/
│       └── otp_client.exe          # Windows 32-bit ✅
├── x86_64-pc-windows-gnu/
│   └── release/
│       └── otp_client.exe          # Windows 64-bit ✅
└── x86_64-unknown-linux-gnu/
    └── release/
        └── otp_client              # Linux 64-bit ✅

server/target/
├── i686-pc-windows-gnu/
│   └── release/
│       └── otp_server.exe          # Windows 32-bit ✅
├── x86_64-pc-windows-gnu/
│   └── release/
│       └── otp_server.exe          # Windows 64-bit ✅
└── x86_64-unknown-linux-gnu/
    └── release/
        └── otp_server              # Linux 64-bit ✅
```

## Рабочие файлы (создаются при использовании)

```
bs1.txt          # Базовый секрет 1 (клиент)
bs3.txt          # Начальная настройка часов (клиент)
database.txt     # База данных пользователей (сервер)
```

## Размеры исполняемых файлов

Приблизительные размеры после сборки:
- **Windows .exe**: ~3-5 MB
- **Linux binary**: ~3-4 MB

Все статически слинкованы, не требуют дополнительных библиотек!

## Зависимости проектов

### client & server
- **eframe** - GUI фреймворк
- **egui** - UI библиотека
- **des** - DES шифрование
- **hex** - HEX кодирование
- **chrono** - Работа с датой/временем
- **encoding_rs** - Поддержка Windows-1251
