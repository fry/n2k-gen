#[derive(Debug)]
pub enum N2kError {
    InvalidPayloadSize {
        pgn: u32,
        expected: usize,
        actual: usize,
    },
    UnknownPgn(u32),
}
