# Команды soundwave-firmware. `make` или `make help` — список.
#
# Окружение Xtensa-тулчейна (PATH, LIBCLANG_PATH) подгружается из файла,
# который создаёт `espup install`, — руками `. ~/export-esp.sh` делать не нужно.
ESP_ENV ?= $(HOME)/export-esp.sh

CORE := cd core &&
ESP  := cd esp && . $(ESP_ENV) &&
ELF  := esp/target/xtensa-esp32-none-elf/release/soundwave

.DEFAULT_GOAL := help
.PHONY: help test build flash monitor fmt lint check core-check esp-check image clean

help: ## показать этот список
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*## "} {printf "  \033[36m%-11s\033[0m %s\n", $$1, $$2}'

# --- Каждый день -------------------------------------------------------------

test: ## тесты core (плата не нужна)
	$(CORE) cargo test

build: ## собрать прошивку
	$(ESP) cargo build --release

flash: ## собрать, прошить и открыть монитор (выход: Ctrl+C)
	$(ESP) cargo run --release

monitor: ## только монитор логов платы
	$(ESP) espflash monitor

fmt: ## отформатировать код обоих крейтов
	$(CORE) cargo fmt
	$(ESP) cargo fmt

lint: ## clippy для обоих крейтов
	$(CORE) cargo clippy --all-targets -- -D warnings
	$(ESP) cargo clippy --release -- -D warnings

# --- То же, что гоняет CI ----------------------------------------------------

check: core-check esp-check ## все проверки CI разом

core-check: ## core: формат, clippy, тесты
	$(CORE) cargo fmt --check
	$(CORE) cargo clippy --all-targets -- -D warnings
	$(CORE) cargo test

esp-check: ## esp: формат, clippy, сборка
	$(ESP) cargo fmt --check
	$(ESP) cargo clippy --release -- -D warnings
	$(ESP) cargo build --release

image: build ## dist/soundwave.bin (шьётся с 0x0) и dist/soundwave.elf
	mkdir -p dist
	cp $(ELF) dist/soundwave.elf
	$(ESP) espflash save-image --chip esp32 --merge --skip-padding ../dist/soundwave.elf ../dist/soundwave.bin

clean: ## удалить всё собранное
	$(CORE) cargo clean
	$(ESP) cargo clean
	rm -rf dist
