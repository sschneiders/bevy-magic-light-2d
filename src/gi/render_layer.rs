

use bevy::prelude::*;

pub struct FloorType;
pub struct WallsType;
pub struct ObjectsType;

pub static MAGIC_LIGHT_2D_FLOOR: usize = 0;
pub static MAGIC_LIGHT_2D_WALLS: usize = 1;
pub static MAGIC_LIGHT_2D_OBJECTS: usize = 2;
pub static MAGIC_LIGHT_2D_LIGHTS: usize = 3;

pub static ALL_LAYERS: [usize; 4] = [0, 1, 2, 3];


pub mod render_layer {
    use super::*;

    pub struct PostProcessingType;

    pub static CAMERA_LAYER_POST_PROCESSING: u8 = 4;

    pub static MAGIC_LIGHT_2D_POST_PROCESSING: usize = 4;

    pub fn floor_layer() -> Visibility {
        Visibility::Visible
    }

    pub fn walls_layer() -> Visibility {
        Visibility::Visible
    }

    pub fn objects_layer() -> Visibility {
        Visibility::Visible
    }

    pub fn lights_layer() -> Visibility {
        Visibility::Visible
    }

    pub fn all_layers() -> Visibility {
        Visibility::Visible
    }

    pub fn post_processing_layer() -> Visibility {
        Visibility::Visible
    }
}

