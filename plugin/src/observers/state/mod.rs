mod account;
mod clock;
mod cron;
mod epoch;
mod now;
mod pyth;
mod slot;
mod webhook;

pub use account::*;
pub use clock::*;
pub use cron::*;
pub use epoch::*;
pub use now::*;
pub use pyth::*;
pub use slot::*;
pub use webhook::*;

use super::thread::ThreadObserver;
use std::sync::atomic::AtomicU64;

pub trait FromState<S> {
    fn from(state: &S) -> &Self;
}

impl FromState<ThreadObserver> for AtomicU64 {
    fn from(state: &ThreadObserver) -> &Self {
        &state.current_epoch
    }
}
