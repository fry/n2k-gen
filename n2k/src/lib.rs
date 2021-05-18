#![no_std]

pub const GLOBAL_ADDRESS: u8 = 0xff;

use embedded_hal_can as hal_can;

mod bus;
pub use bus::{Bus, BusError};

mod id;
pub use id::{Id, IdError, Priority};

mod message;
pub use message::Message;

mod name;
pub use name::Name;

mod product;
pub use product::Product;

mod frame;
pub use frame::CanFrame;

mod fast_packet;

pub trait PgnRegistry {
    type Message;
    type Error;

    // fn is_known(pgn: u32) -> bool;
    fn is_fast_packet(pgn: u32) -> bool;
    fn build_message(pgn: u32, data: &[u8]) -> Result<Self::Message, Self::Error>;
}
