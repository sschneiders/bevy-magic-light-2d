use bevy::prelude::*;

pub const GI_SCREEN_PROBE_SIZE: i32 = 8;

/// Resource to store strong handles for post-processing assets
#[derive(Resource)]
pub struct PostProcessingHandles {
    pub rect_mesh: Option<Handle<Mesh>>,
    pub material: Option<Handle<crate::gi::compositing::PostProcessingMaterial>>,
}

impl Default for PostProcessingHandles {
    fn default() -> Self {
        Self {
            rect_mesh: None,
            material: None,
        }
    }
}
