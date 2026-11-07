# soundwave-firmware

Прошивка ESP32 для soundwave — подсветки стола в такт Apple Music. Принимает с мака цвета и ритм и рисует эффекты
на ленте WS2812B.

Bare-metal Rust на [esp-hal](https://github.com/esp-rs/esp-hal) + [embassy](https://embassy.dev):
без ОС и без `std`, есть только `core`, периферия чипа и async-планировщик.
Каркас сгенерирован `esp-generate` (`-o esp32-wroom-32 -o unstable-hal -o embassy -o log`).

## Структура

Два независимых крейта:

| Папка | Крейт | Что внутри | Собирается |
|---|---|---|---|
| `core/` | `soundwave-core` | логика без железа: эффекты, расчёт тока, позже — разбор пакетов. Тут же тесты | обычным stable Rust, под мак |
| `esp/` | `soundwave` | сама прошивка: железо, Wi-Fi, лента. Зависит от `core` | тулчейном `esp` под Xtensa |

Всё, что можно проверить без платы, живёт в `core` и покрыто тестами.

## Установка тулчейна (один раз)

```sh
cargo install espup esp-generate espflash --locked
espup install --targets esp32       # Rust-тулчейн для Xtensa (архитектура ESP32)
```

## Работа

Тесты (плата не нужна):

```sh
cd core
cargo test
```

Прошивка. В каждом новом терминале сначала подгрузи окружение тулчейна:

```sh
cd esp
. ~/export-esp.sh
cargo build --release     # просто собрать
cargo run --release       # собрать, прошить, открыть монитор (выход: Ctrl+C)
espflash monitor          # только монитор логов
```

`cargo run` сам вызывает `espflash flash --monitor` (см. `esp/.cargo/config.toml`) и сам находит порт.

> Прошивка затрёт WLED. Вернуть его можно в любой момент через веб-установщик install.wled.me.

## Что где в `esp/`

| Файл | Зачем |
|---|---|
| `src/bin/main.rs` | сама прошивка |
| `Cargo.toml` | зависимости: `esp-hal` (железо), `esp-rtos` + `embassy-*` (async), `esp-hal-smartled` (лента через RMT) |
| `.cargo/config.toml` | цель `xtensa-esp32-none-elf`, сборка `core` из исходников, runner |
| `rust-toolchain.toml` | использовать тулчейн `esp` от espup |
| `build.rs` | подключает скрипт линковщика `linkall.x` (раскладка памяти чипа) |
