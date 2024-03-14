mod handlers;
mod types;

pub use handlers::{
    init_broadcast, login_user, logout_user, refresh_user, register_user, subscribe_to_broadcast,
};
pub use types::ApplicationState;

pub fn welcome() {
    println!("Welcome to the LiveScript API!");
}
