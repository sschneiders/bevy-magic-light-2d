use bevy::prelude::*;

pub const GI_SCREEN_PROBE_SIZE: i32 = 8;

/// Resource to store strong handles for post-processing assets
#[derive(Resource)]
pub struct PostProcessingHandles {
    pub rect_mesh: Handle<Mesh>,
    pub material: Handle<crate::gi::compositing::PostProcessingMaterial>,
}

impl Default for PostProcessingHandles {
    fn default() -> Self {
        Self {
            rect_mesh: Handle::default(),
            material: Handle::default(),
        }
    }
}
