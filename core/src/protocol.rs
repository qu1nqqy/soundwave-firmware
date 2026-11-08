//! Протокол обмена с маком: один UDP-пакет — одна команда.
//!
//! Формат v1:
//!
//! | Байт    | Значение            |
//! |---------|---------------------|
//! | 0       | [`MAGIC`] (`'S'`)   |
//! | 1       | [`VERSION`]         |
//! | 2       | тип команды         |
//! | 3..     | данные команды      |
//!
//! Команды:
//!
//! - [`CMD_FILL`]: данные — `R G B`, всего пакет 6 байт.

use smart_leds::RGB8;

/// UDP-порт, который слушает плата.
pub const PORT: u16 = 4242;
/// Первый байт каждого пакета: отсеивает случайный мусор.
pub const MAGIC: u8 = b'S';
/// Версия протокола, которую понимает прошивка.
pub const VERSION: u8 = 1;
/// Длина шапки: магия + версия + тип команды.
pub const HEADER_LEN: usize = 3;

/// «Залей всю ленту цветом».
pub const CMD_FILL: u8 = 0x01;

/// Команда, которую прислал мак.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Залить всю ленту одним цветом.
    Fill(RGB8),
}

/// Почему пакет не удалось разобрать.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Пакет короче шапки.
    TooShort,
    /// Первый байт — не [`MAGIC`].
    BadMagic,
    /// Версия протокола, которую прошивка не знает.
    UnsupportedVersion(u8),
    /// Тип команды, которого нет в протоколе.
    UnknownCommand(u8),
    /// Шапка в порядке, но длина данных не подходит для этой команды.
    BadLength,
}

/// Разбирает UDP-пакет в команду.
///
/// Проверки идут по порядку: длина шапки → магия → версия → тип команды → длина данных.
/// Первая же неудачная проверка и становится ошибкой.
pub fn parse(packet: &[u8]) -> Result<Command, ParseError> {
    if packet.len() < HEADER_LEN {
        return Err(ParseError::TooShort);
    }
    if packet[0] != MAGIC {
        return Err(ParseError::BadMagic);
    }
    if packet[1] != VERSION {
        return Err(ParseError::UnsupportedVersion(packet[1]));
    }
    match packet[2] {
        CMD_FILL => match &packet[HEADER_LEN..] {
            &[r, g, b] => Ok(Command::Fill(RGB8::new(r, g, b))),
            _ => Err(ParseError::BadLength),
        },
        other => Err(ParseError::UnknownCommand(other)),
    }
}

#[cfg(test)]
mod tests;
