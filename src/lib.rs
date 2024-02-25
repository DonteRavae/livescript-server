mod handlers;
mod types;

pub use handlers::{init_broadcast, subscribe_to_broadcast};
pub use types::ApplicationState;

pub fn welcome() {
    println!("Welcome to the LiveScript API!");
}
