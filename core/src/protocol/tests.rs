use super::*;

fn fill(r: u8, g: u8, b: u8) -> [u8; 6] {
    [MAGIC, VERSION, CMD_FILL, r, g, b]
}

// --- Правильные пакеты ---------------------------------------------------

#[test]
fn fill_red() {
    assert_eq!(
        parse(&fill(255, 0, 0)),
        Ok(Command::Fill(RGB8::new(255, 0, 0)))
    );
}

#[test]
fn fill_keeps_channel_order() {
    // R, G, B идут именно в таком порядке — перепутать легко.
    assert_eq!(parse(&fill(1, 2, 3)), Ok(Command::Fill(RGB8::new(1, 2, 3))));
}

#[test]
fn fill_black() {
    assert_eq!(parse(&fill(0, 0, 0)), Ok(Command::Fill(RGB8::new(0, 0, 0))));
}

#[test]
fn same_bytes_as_the_nc_example() {
    // printf 'S\x01\x01\xff\x00\x00' | nc -u -w1 <ip> 4242
    assert_eq!(
        parse(b"S\x01\x01\xff\x00\x00"),
        Ok(Command::Fill(RGB8::new(255, 0, 0)))
    );
}

// --- Битая шапка -----------------------------------------------------------

#[test]
fn empty_packet_is_too_short() {
    assert_eq!(parse(&[]), Err(ParseError::TooShort));
}

#[test]
fn packet_shorter_than_header_is_too_short() {
    assert_eq!(parse(&[MAGIC, VERSION]), Err(ParseError::TooShort));
}

#[test]
fn wrong_magic() {
    assert_eq!(
        parse(&[b'X', VERSION, CMD_FILL, 0, 0, 0]),
        Err(ParseError::BadMagic)
    );
}

#[test]
fn unsupported_version() {
    assert_eq!(
        parse(&[MAGIC, 2, CMD_FILL, 0, 0, 0]),
        Err(ParseError::UnsupportedVersion(2))
    );
}

#[test]
fn unknown_command() {
    assert_eq!(
        parse(&[MAGIC, VERSION, 0x7f, 0, 0, 0]),
        Err(ParseError::UnknownCommand(0x7f))
    );
}

// --- Порядок проверок --------------------------------------------------------

#[test]
fn magic_is_checked_before_version() {
    assert_eq!(parse(&[b'X', 2, 0x7f]), Err(ParseError::BadMagic));
}

#[test]
fn version_is_checked_before_command() {
    assert_eq!(
        parse(&[MAGIC, 2, 0x7f]),
        Err(ParseError::UnsupportedVersion(2))
    );
}

#[test]
fn command_is_checked_before_length() {
    // Неизвестная команда без данных — это всё равно «неизвестная команда».
    assert_eq!(
        parse(&[MAGIC, VERSION, 0x7f]),
        Err(ParseError::UnknownCommand(0x7f))
    );
}

// --- Длина данных ------------------------------------------------------------

#[test]
fn fill_without_color() {
    assert_eq!(
        parse(&[MAGIC, VERSION, CMD_FILL]),
        Err(ParseError::BadLength)
    );
}

#[test]
fn fill_missing_blue() {
    assert_eq!(
        parse(&[MAGIC, VERSION, CMD_FILL, 1, 2]),
        Err(ParseError::BadLength)
    );
}

#[test]
fn fill_with_extra_byte() {
    assert_eq!(
        parse(&[MAGIC, VERSION, CMD_FILL, 1, 2, 3, 4]),
        Err(ParseError::BadLength)
    );
}
