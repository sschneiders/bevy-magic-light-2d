use bevy::asset::uuid_handle;
use bevy::prelude::*;

use crate::gi::compositing::PostProcessingMaterial;

pub const GI_SCREEN_PROBE_SIZE: i32 = 8;

pub const POST_PROCESSING_RECT: Handle<Mesh> = uuid_handle!("9999c9b9-c46a-48e7-b7b8-023a354b7cac");
pub const POST_PROCESSING_MATERIAL: Handle<PostProcessingMaterial> = uuid_handle!("9999c9b9-c46a-48e7-b7b8-023a354b9cac");
