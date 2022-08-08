use crate::prelude::*;

mod builder;
mod events;
mod logstore;
pub use builder::*;
pub use events::*;
use logstore::*;
pub use logstore::{clear_log, clone_log, log_display, restore_log};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}
