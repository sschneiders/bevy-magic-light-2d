
use bevy::prelude::*;

fn main() {
    println!("Testing material imports in Bevy 0.17");
    
    // Let's try to access Material2d directly
    App::new()
        .add_plugins(DefaultPlugins)
        .run();
}
