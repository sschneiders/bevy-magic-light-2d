use bevy::prelude::*;
use bevy::render::render_resource::{StorageBuffer, UniformBuffer};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::Extract;
use rand::Rng;

use crate::gi::constants::GI_SCREEN_PROBE_SIZE;
use crate::gi::resource::ComputedTargetSizes;
use crate::gi::types::{LightOccluder2D, OmniLightSource2D, SkylightLight2D, SkylightMask2D};
use crate::gi::types_gpu::{
    GpuCameraParams,
    GpuLightOccluder2D,
    GpuLightOccluderBuffer,
    GpuLightPassParams,
    GpuLightSourceBuffer,
    GpuOmniLightSource,
    GpuProbeDataBuffer,
    GpuSkylightMaskBuffer,
    GpuSkylightMaskData,
};
use crate::prelude::BevyMagicLight2DSettings;
use crate::FloorCamera;

#[rustfmt::skip]
// With load_shader_library!, embedded shader dependencies are handled automatically
// This resource is kept for compatibility but no longer needed for manual preloading
#[derive(Default, Resource)]
pub(crate) struct EmbeddedShaderDependencies {
    // No longer tracking loaded shaders manually - load_shader_library! handles it
}

#[rustfmt::skip]
// With load_shader_library!, we don't need to manually preload shader dependencies
// The macro handles embedding and loading automatically when needed
pub(crate) fn system_load_embedded_shader_dependencies(
    mut _embedded_shader_deps: ResMut<EmbeddedShaderDependencies>,
    _asset_server: Res<AssetServer>,
) {
    // Shaders are automatically loaded by load_shader_library! macro
    // Manual preloading is no longer needed with Bevy 0.17's improved shader loading
}

pub(crate) fn load_embedded_shader(asset_server: &AssetServer, shader_file: &str)
    -> Handle<Shader>
{
    // With load_shader_library!, shaders are embedded and loaded using embedded:// protocol
    // Try the embedded protocol format that should work with the macro
    asset_server.load(format!("embedded://bevy_magic_light_2d/gi/shaders/{}", shader_file))
}

#[rustfmt::skip]
#[derive(Default, Resource)]
pub struct LightPassPipelineAssets {
    pub camera_params:     UniformBuffer<GpuCameraParams>,
    pub light_pass_params: UniformBuffer<GpuLightPassParams>,
    pub light_sources:     StorageBuffer<GpuLightSourceBuffer>,
    pub light_occluders:   StorageBuffer<GpuLightOccluderBuffer>,
    pub probes:            StorageBuffer<GpuProbeDataBuffer>,
    pub skylight_masks:    StorageBuffer<GpuSkylightMaskBuffer>,
}

impl LightPassPipelineAssets
{
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue)
    {
        self.light_sources.write_buffer(device, queue);
        self.light_occluders.write_buffer(device, queue);
        self.camera_params.write_buffer(device, queue);
        self.light_pass_params.write_buffer(device, queue);
        self.probes.write_buffer(device, queue);
        self.skylight_masks.write_buffer(device, queue);
    }
}

#[rustfmt::skip]
pub fn system_prepare_pipeline_assets(
    render_device:         Res<RenderDevice>,
    render_queue:          Res<RenderQueue>,
    mut gi_compute_assets: ResMut<LightPassPipelineAssets>,
) {
    gi_compute_assets.write_buffer(&render_device, &render_queue);
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn system_extract_pipeline_assets(
    res_light_settings:         Extract<Res<BevyMagicLight2DSettings>>,
    res_target_sizes:           Extract<Res<ComputedTargetSizes>>,

    query_lights:               Extract<Query<(&GlobalTransform, &OmniLightSource2D, &InheritedVisibility, &ViewVisibility)>>,
    query_occluders:            Extract<Query<(&LightOccluder2D, &GlobalTransform, &Transform, &InheritedVisibility, &ViewVisibility)>>,
    query_camera:               Extract<Query<(&Camera, &GlobalTransform), With<FloorCamera>>>,
    query_masks:                Extract<Query<(&GlobalTransform, &SkylightMask2D)>>,
    query_skylight_light:       Extract<Query<&SkylightLight2D>>,

    mut gpu_target_sizes:       ResMut<ComputedTargetSizes>,
    mut gpu_pipeline_assets:    ResMut<LightPassPipelineAssets>,
    mut gpu_frame_counter:      Local<i32>,
    mut prev_view_proj:         Local<Mat4>,
    mut prev_camera_scale:      Local<f32>,
    mut prev_camera_translation: Local<Vec3>,
) {
    let light_pass_config = &res_light_settings.light_pass_params;

    *gpu_target_sizes = **res_target_sizes;

    // Initialize previous camera tracking if this is the first frame
    if !prev_view_proj.is_finite() && *prev_camera_scale == 0.0 {
        if let Ok((camera, camera_global_transform)) = query_camera.single() {
            *prev_view_proj = camera.clip_from_view(); // Just use the projection for initialization
            *prev_camera_translation = camera_global_transform.translation();
            *prev_camera_scale = camera.clip_from_view().col(0).x;
        }
    }

    {
        let light_sources = gpu_pipeline_assets.light_sources.get_mut();
        let mut rng = rand::rng();
        light_sources.count = 0;
        light_sources.data.clear();
        for (transform, light_source, hviz, vviz) in query_lights.iter() {
            if hviz.get() && vviz.get() {
                light_sources.count += 1;
                light_sources.data.push(GpuOmniLightSource::new(
                    OmniLightSource2D {
                        intensity: light_source.intensity
                            + rng.random_range(-1.0..1.0) * light_source.jitter_intensity,
                        ..*light_source
                    },
                    Vec2::new(
                        transform.translation().x
                            + rng.random_range(-1.0..1.0) * light_source.jitter_translation,
                        transform.translation().y
                            + rng.random_range(-1.0..1.0) * light_source.jitter_translation,
                    ),
                ));
            }
        }
    }

    {
        let light_occluders = gpu_pipeline_assets.light_occluders.get_mut();
        light_occluders.count = 0;
        light_occluders.data.clear();
        for (occluder, global_transform, transform, hviz, vviz) in query_occluders.iter() {
            if hviz.get() && vviz.get() {
                light_occluders.count += 1;
                light_occluders.data.push(GpuLightOccluder2D {
                    center: global_transform.translation().xy(),
                    rotation: transform.rotation.inverse().into(),
                    h_extent: occluder.h_size,
                });
            }
        }
    }

    {
        let skylight_masks = gpu_pipeline_assets.skylight_masks.get_mut();
        skylight_masks.count = 0;
        skylight_masks.data.clear();
        for (transform, mask) in query_masks.iter() {
            skylight_masks.count += 1;
            skylight_masks.data.push(GpuSkylightMaskData::new(
                transform.translation().truncate(),
                mask.h_size,
            ));
        }
    }

    {
        if let Ok((camera, camera_global_transform)) = query_camera.single() {
            let camera_params = gpu_pipeline_assets.camera_params.get_mut();
            let projection = camera.clip_from_view();
            let inverse_projection = projection.inverse();
            let view = camera_global_transform.to_matrix();
            let inverse_view = view.inverse();
            
            let current_view_proj = projection * inverse_view;
            
            // Detect camera zoom/projection changes
            let current_scale = projection.col(0).x; // For orthographic cameras, this represents the zoom level
            let projection_change = if prev_view_proj.is_finite() {
                // Check for significant changes in projection matrix
                let view_proj_diff = (current_view_proj - *prev_view_proj).abs();
                let scale_diff = (current_scale - *prev_camera_scale).abs();
                
                // Calculate maximum absolute difference across all matrix elements
                let max_projection_diff = view_proj_diff.to_cols_array().into_iter().fold(0.0f32, |acc, x| acc.max(x));
                
                // Much more sensitive thresholds for detecting zoom changes
                let zoom_threshold = 0.001;  // Very sensitive - 0.1% scale change
                let projection_threshold = 0.01;  // More sensitive projection changes
                
                // Detect any camera movement or scaling
                let camera_movement = (camera_global_transform.translation() - *prev_camera_translation).length_squared();
                let camera_movement_threshold = 0.01; // Sensitive to movement as well
                
                // If camera moved significantly or projection changed, trigger temporal reset
                if camera_movement > camera_movement_threshold {
                    log::debug!("Camera movement detected: movement={}, triggering temporal reset", camera_movement.sqrt());
                    1.0 // Reset temporal accumulation
                } else if scale_diff > zoom_threshold {
                    log::debug!("Zoom change detected: scale_diff={}, triggering temporal reset", scale_diff);
                    1.0 // Reset temporal accumulation
                } else if max_projection_diff > projection_threshold {
                    log::debug!("Projection change detected: max_diff={}, triggering temporal reset", max_projection_diff);
                    1.0 // Reset temporal accumulation
                } else {
                    0.0 // Normal temporal accumulation
                }
            } else {
                0.0 // First frame, no reset needed
            };
            
            // Update previous frame values
            *prev_view_proj = current_view_proj;
            *prev_camera_scale = current_scale;
            *prev_camera_translation = camera_global_transform.translation();

            camera_params.view_proj = current_view_proj;
            camera_params.inverse_view_proj = view * inverse_projection;
            camera_params.screen_size = Vec2::new(
                gpu_target_sizes.primary_target_size.x,
                gpu_target_sizes.primary_target_size.y,
            );
            camera_params.screen_size_inv = Vec2::new(
                1.0 / gpu_target_sizes.primary_target_size.x,
                1.0 / gpu_target_sizes.primary_target_size.y,
            );

            let scale = 2.0;
            camera_params.sdf_scale     = Vec2::splat(scale);
            camera_params.inv_sdf_scale = Vec2::splat(1. / scale);
            camera_params.temporal_reset = projection_change;

            let probes = gpu_pipeline_assets.probes.get_mut();
            probes.data[*gpu_frame_counter as usize].camera_pose =
                camera_global_transform.translation().truncate();
                
            // Reset frame counter during zoom to break temporal sampling completely
            if projection_change > 0.5 {
                *gpu_frame_counter = 0;
                log::debug!("Reset frame counter due to temporal reset");
            }
        } else {
            log::warn!("Failed to get camera");
            let camera_params = gpu_pipeline_assets.camera_params.get_mut();
            camera_params.temporal_reset = 0.0;
            
            let probes = gpu_pipeline_assets.probes.get_mut();
            probes.data[*gpu_frame_counter as usize].camera_pose = Vec2::ZERO;
        }
    }

    {
        let light_pass_params = gpu_pipeline_assets.light_pass_params.get_mut();
        light_pass_params.frame_counter = *gpu_frame_counter;
        light_pass_params.probe_size = GI_SCREEN_PROBE_SIZE;
        light_pass_params.probe_atlas_cols            = gpu_target_sizes.probe_grid_isize.x;
        light_pass_params.probe_atlas_rows            = gpu_target_sizes.probe_grid_isize.y;
        light_pass_params.reservoir_size              = light_pass_config.reservoir_size;
        light_pass_params.smooth_kernel_size_h        = light_pass_config.smooth_kernel_size.0;
        light_pass_params.smooth_kernel_size_w        = light_pass_config.smooth_kernel_size.1;
        light_pass_params.direct_light_contrib        = light_pass_config.direct_light_contrib;
        light_pass_params.indirect_light_contrib      = light_pass_config.indirect_light_contrib;
        light_pass_params.indirect_rays_radius_factor = light_pass_config.indirect_rays_radius_factor;
        light_pass_params.indirect_rays_per_sample    = light_pass_config.indirect_rays_per_sample;
    }

    {
        let light_pass_params = gpu_pipeline_assets.light_pass_params.get_mut();
        light_pass_params.skylight_color = Vec3::splat(0.0);
        for new_gi_state in query_skylight_light.iter() {
            let srgba = new_gi_state.color.to_srgba();
            light_pass_params.skylight_color.x += srgba.red * new_gi_state.intensity;
            light_pass_params.skylight_color.y += srgba.green * new_gi_state.intensity;
            light_pass_params.skylight_color.z += srgba.blue * new_gi_state.intensity;
        }
    }

    *gpu_frame_counter = (*gpu_frame_counter + 1) % (GI_SCREEN_PROBE_SIZE * GI_SCREEN_PROBE_SIZE);
}
