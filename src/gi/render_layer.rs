use bevy::camera::visibility::Layer;

pub const CAMERA_LAYER_FLOOR: Layer = 1;
pub const CAMERA_LAYER_WALLS: Layer = 2;
pub const CAMERA_LAYER_OBJECTS: Layer = 3;


pub const ALL_LAYERS: &[Layer] = &[CAMERA_LAYER_FLOOR, CAMERA_LAYER_WALLS, CAMERA_LAYER_OBJECTS];

pub const CAMERA_LAYER_POST_PROCESSING: Layer = 42;
