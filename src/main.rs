pub mod math;
pub mod orbit;
pub mod time;

use bevy::{app::App, DefaultPlugins};

fn main() {
    App::new().add_plugins(DefaultPlugins).run()
}
