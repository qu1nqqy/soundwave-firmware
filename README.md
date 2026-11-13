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

## Wi-Fi

Имя сети и пароль в репо не хранятся. Один раз создай локальный файл и впиши свою сеть:

```sh
cp esp/.cargo/wifi.toml.example esp/.cargo/wifi.toml
```

Без него сборка прошивки падает с `failed to load config include wifi.toml`.

## Работа

Всё через `make` из корня репо. Окружение тулчейна (`~/export-esp.sh`) Makefile подгружает сам.

```sh
make            # список команд
make test       # тесты core (плата не нужна)
make build      # собрать прошивку
make flash      # собрать, прошить, открыть монитор (выход: Ctrl+C)
make monitor    # только монитор логов
make fmt        # отформатировать код
make check      # все проверки, как в CI
```

`make flash` вызывает `cargo run`, а тот — `espflash flash --monitor` (см. `esp/.cargo/config.toml`),
порт находится сам. Если espup положил файл окружения в другое место: `make flash ESP_ENV=/путь/к/export-esp.sh`.

> Прошивка затрёт WLED. Вернуть его можно в любой момент через веб-установщик install.wled.me.

## CI/CD

`.github/workflows/build.yml`:

- **каждый пуш в `main`/`dev` и PR в `main`:** `make core-check` и `make esp-check` — те же команды, что и локально.
- **пуш в `main`:** ещё и релиз `v<версия>` с файлами `soundwave.bin` и `soundwave.elf`.
  Версия берётся из `esp/Cargo.toml`; если такой релиз уже есть, CI падает ещё в PR — нужно поднять версию.
- **уведомления в Telegram** через [tg-notify](https://github.com/qu1nqqy/tg-notify): одно сообщение на запуск,
  обновляется по мере шагов, звук только при провале; о релизе — отдельное закреплённое сообщение с файлами.
  Нужны секреты `TELEGRAM_TOKEN`, `TELEGRAM_CHAT_ID` и переменная `TELEGRAM_TOPIC_ID`. Без них CI работает,
  просто молча.

Прошить готовый релиз без сборки:

```sh
espflash write-bin 0x0 soundwave.bin
```

## Что где в `esp/`

| Файл | Зачем |
|---|---|
| `src/bin/main.rs` | сама прошивка |
| `Cargo.toml` | зависимости: `esp-hal` (железо), `esp-rtos` + `embassy-*` (async), `esp-hal-smartled` (лента через RMT) |
| `.cargo/config.toml` | цель `xtensa-esp32-none-elf`, сборка `core` из исходников, runner |
| `rust-toolchain.toml` | использовать тулчейн `esp` от espup |
| `build.rs` | подключает скрипт линковщика `linkall.x` (раскладка памяти чипа) |
