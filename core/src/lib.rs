//! Логика soundwave, которая не зависит от железа: эффекты, расчёт тока,
//! позже — разбор UDP-пакетов.
//!
//! Крейт собирается и под ESP32 (`no_std`), и под мак, поэтому его можно
//! тестировать обычным `cargo test` без платы.

#![cfg_attr(not(test), no_std)]

pub mod effects;
pub mod power;
pub mod protocol;
