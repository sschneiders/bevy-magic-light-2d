use bevy::prelude::*;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::asset::RenderAssetUsages;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::GpuImage;

use crate::gi::pipeline_assets::{load_embedded_shader, LightPassPipelineAssets};
use crate::gi::resource::ComputedTargetSizes;
use crate::gi::types_gpu::{
    GpuCameraParams,
    GpuLightOccluderBuffer,
    GpuLightPassParams,
    GpuLightSourceBuffer,
    GpuProbeDataBuffer,
    GpuSkylightMaskBuffer,
};

const SDF_TARGET_FORMAT: TextureFormat = TextureFormat::R16Float;
const SS_PROBE_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
const SS_BOUNCE_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
const SS_BLEND_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
const SS_FILTER_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
const SS_POSE_TARGET_FORMAT: TextureFormat = TextureFormat::Rg32Float;

const SDF_PIPELINE_ENTRY: &str = "main";
const SS_PROBE_PIPELINE_ENTRY: &str = "main";
const SS_BOUNCE_PIPELINE_ENTRY: &str = "main";
const SS_BLEND_PIPELINE_ENTRY: &str = "main";
const SS_FILTER_PIPELINE_ENTRY: &str = "main";

#[allow(dead_code)]
#[derive(Clone, Resource, ExtractResource, Default)]
pub struct GiTargetsWrapper
{
    pub targets: Option<GiTargets>,
}

#[derive(Clone)]
pub struct GiTargets
{
    pub sdf_target:       Handle<Image>,
    pub ss_probe_target:  Handle<Image>,
    pub ss_bounce_target: Handle<Image>,
    pub ss_blend_target:  Handle<Image>,
    pub ss_filter_target: Handle<Image>,
    pub ss_pose_target:   Handle<Image>,
}

impl GiTargets
{
    pub fn create(images: &mut Assets<Image>, sizes: &ComputedTargetSizes) -> Self
    {
        let sdf_tex = create_texture_2d(
            sizes.sdf_target_usize.into(),
            SDF_TARGET_FORMAT,
            ImageFilterMode::Linear,
        );
        let ss_probe_tex = create_texture_2d(
            sizes.primary_target_usize.into(),
            SS_PROBE_TARGET_FORMAT,
            ImageFilterMode::Nearest,
        );
        let ss_bounce_tex = create_texture_2d(
            sizes.primary_target_usize.into(),
            SS_BOUNCE_TARGET_FORMAT,
            ImageFilterMode::Nearest,
        );
        let ss_blend_tex = create_texture_2d(
            sizes.probe_grid_usize.into(),
            SS_BLEND_TARGET_FORMAT,
            ImageFilterMode::Nearest,
        );
        let ss_filter_tex = create_texture_2d(
            sizes.primary_target_usize.into(),
            SS_FILTER_TARGET_FORMAT,
            ImageFilterMode::Nearest,
        );
        let ss_pose_tex = create_texture_2d(
            sizes.primary_target_usize.into(),
            SS_POSE_TARGET_FORMAT,
            ImageFilterMode::Nearest,
        );

        let sdf_target: Handle<Image> = images.reserve_handle();
        let ss_probe_target: Handle<Image> = images.reserve_handle();
        let ss_bounce_target: Handle<Image> = images.reserve_handle();
        let ss_blend_target: Handle<Image> = images.reserve_handle();
        let ss_filter_target: Handle<Image> = images.reserve_handle();
        let ss_pose_target: Handle<Image> = images.reserve_handle();

        let _ = images.insert(sdf_target.id(), sdf_tex);
        let _ = images.insert(ss_probe_target.id(), ss_probe_tex);
        let _ = images.insert(ss_bounce_target.id(), ss_bounce_tex);
        let _ = images.insert(ss_blend_target.id(), ss_blend_tex);
        let _ = images.insert(ss_filter_target.id(), ss_filter_tex);
        let _ = images.insert(ss_pose_target.id(), ss_pose_tex);

        Self {
            sdf_target,
            ss_probe_target,
            ss_bounce_target,
            ss_blend_target,
            ss_filter_target,
            ss_pose_target,
        }
    }
}

#[allow(dead_code)]
#[derive(Resource)]
pub struct LightPassPipelineBindGroups
{
    pub sdf_bind_group:       BindGroup,
    pub ss_blend_bind_group:  BindGroup,
    pub ss_probe_bind_group:  BindGroup,
    pub ss_bounce_bind_group: BindGroup,
    pub ss_filter_bind_group: BindGroup,
}

#[rustfmt::skip]
fn create_texture_2d(size: (u32, u32), format: TextureFormat, filter: ImageFilterMode) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.0,
            height: size.1,
            ..Default::default()
        },
        TextureDimension::D2,
        &[
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        format,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: filter,
        min_filter: filter,
        address_mode_u: ImageAddressMode::ClampToBorder,
        address_mode_v: ImageAddressMode::ClampToBorder,
        address_mode_w: ImageAddressMode::ClampToBorder,
        ..Default::default()
    });

    image
}

#[rustfmt::skip]
pub fn system_setup_gi_pipeline(
    mut images:          ResMut<Assets<Image>>,
    mut targets_wrapper: ResMut<GiTargetsWrapper>,
    targets_sizes:   Res<ComputedTargetSizes>,
) {
    targets_wrapper.targets = Some(GiTargets::create(&mut images, &targets_sizes));
}

#[derive(Resource)]
pub struct LightPassPipeline
{
    pub sdf_bind_group_layout:       BindGroupLayout,
    pub sdf_pipeline:                CachedComputePipelineId,
    pub ss_probe_bind_group_layout:  BindGroupLayout,
    pub ss_probe_pipeline:           CachedComputePipelineId,
    pub ss_bounce_bind_group_layout: BindGroupLayout,
    pub ss_bounce_pipeline:          CachedComputePipelineId,
    pub ss_blend_bind_group_layout:  BindGroupLayout,
    pub ss_blend_pipeline:           CachedComputePipelineId,
    pub ss_filter_bind_group_layout: BindGroupLayout,
    pub ss_filter_pipeline:          CachedComputePipelineId,
}

/// Check if all required GPU buffers are initialized and ready for binding
/// 
/// This function implements a robust validation pattern for Bevy 0.17.2 render resources
/// to prevent race conditions during startup where GPU buffers may not be ready yet.
/// 
/// ## Background
/// In Bevy's render pipeline, resources can be uninitialized for the first few frames
/// due to asynchronous GPU resource loading. The original code would log warnings
/// during normal startup, which was confusing for users.
/// 
/// ## Solution
/// This function provides comprehensive validation:
/// - Checks all storage/uniform buffers are bound
/// - Validates all texture targets are loaded in GPU memory  
/// - Returns specific error messages for debugging
/// - Only proceeds when all resources are ready
/// 
/// This pattern ensures that render passes only execute when all required resources
/// are available, eliminating startup warnings while maintaining robustness.
fn are_buffers_ready(
    gi_compute_assets: &LightPassPipelineAssets,
    gpu_images: &RenderAssets<GpuImage>,
    targets: &GiTargets,
) -> Result<(), String> {
    // Check buffer binding availability
    let bindings = [
        ("light_sources", gi_compute_assets.light_sources.binding()),
        ("light_occluders", gi_compute_assets.light_occluders.binding()),
        ("camera_params", gi_compute_assets.camera_params.binding()),
        ("light_pass_params", gi_compute_assets.light_pass_params.binding()),
        ("probes", gi_compute_assets.probes.binding()),
        ("skylight_masks", gi_compute_assets.skylight_masks.binding()),
    ];

    // Check each buffer
    for (name, binding) in bindings.iter() {
        if binding.is_none() {
            return Err(format!("GPU buffer '{}' not bound yet", name));
        }
    }

    // Check texture targets are loaded in GPU
    let required_targets = [
        ("sdf_target", &targets.sdf_target),
        ("ss_probe_target", &targets.ss_probe_target),
        ("ss_bounce_target", &targets.ss_bounce_target),
        ("ss_blend_target", &targets.ss_blend_target),
        ("ss_filter_target", &targets.ss_filter_target),
        ("ss_pose_target", &targets.ss_pose_target),
    ];

    for (name, target) in required_targets.iter() {
        if gpu_images.get(*target).is_none() {
            return Err(format!("Texture target '{}' not loaded in GPU", name));
        }
    }

    Ok(())
}

/// Queue bind groups for the GI pipeline compute passes
/// 
/// This function is responsible for creating GPU bind groups that connect
/// uniform/storage buffers and textures to the compute shaders.
/// 
/// ## What this does:
/// 1. Validates all GPU resources are ready before proceeding
/// 2. Creates bind groups for SDF, Probe, Bounce, Blend, and Filter passes
/// 3. Inserts the bind groups as a resource for compute shader execution
/// 
/// ## Key improvements for Bevy 0.17.2:
/// - **Graceful startup**: No more warnings during normal initialization
/// - **Robust validation**: Comprehensive resource checking before bind group creation
/// - **Debug logging**: Detailed error messages only at debug level
/// - **Safe unwrapping**: All resources validated before use
/// 
/// ## Error handling:
/// - Early returns when resources aren't ready (normal during startup)
/// - Debug-level logging instead of warnings
/// - No panic scenarios from uninitialized resources
pub fn system_queue_bind_groups(
    mut commands: Commands,
    pipeline: Res<LightPassPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    targets_wrapper: Res<GiTargetsWrapper>,
    gi_compute_assets: Res<LightPassPipelineAssets>,
    render_device: Res<RenderDevice>,
)
{
    // Check if targets are initialized
    let targets = match targets_wrapper.targets.as_ref() {
        Some(targets) => targets,
        None => {
            // This is normal during the first few frames
            return;
        }
    };

    // Validate all buffers and textures are ready before proceeding
    // This prevents the startup warnings that were confusing users
    if let Err(error) = are_buffers_ready(&gi_compute_assets, &gpu_images, targets) {
        // Log at info level for now to debug the black screen issue
        log::info!("GI pipeline resources not ready: {}", error);
        return;
    }

    // Unwrap all bindings now that we know they're ready
    // This is safe because are_buffers_ready() validated everything
    let light_sources = gi_compute_assets.light_sources.binding().unwrap();
    let light_occluders = gi_compute_assets.light_occluders.binding().unwrap();
    let camera_params = gi_compute_assets.camera_params.binding().unwrap();
    let gi_state = gi_compute_assets.light_pass_params.binding().unwrap();
    let probes = gi_compute_assets.probes.binding().unwrap();
    let skylight_masks = gi_compute_assets.skylight_masks.binding().unwrap();

    // Get all texture views - safe to unwrap after validation
    // These are required for the compute shader texture bindings
    let sdf_view_image = gpu_images.get(&targets.sdf_target).unwrap();
    let ss_probe_image = gpu_images.get(&targets.ss_probe_target).unwrap();
    let ss_bounce_image = gpu_images.get(&targets.ss_bounce_target).unwrap();
    let ss_blend_image = gpu_images.get(&targets.ss_blend_target).unwrap();
    let ss_filter_image = gpu_images.get(&targets.ss_filter_target).unwrap();
    let ss_pose_image = gpu_images.get(&targets.ss_pose_target).unwrap();

    // Create all bind groups
    let sdf_bind_group = render_device.create_bind_group(
        "gi_sdf_bind_group",
        &pipeline.sdf_bind_group_layout,
        &[
            BindGroupEntry {
                binding:  0,
                resource: camera_params.clone(),
            },
            BindGroupEntry {
                binding:  1,
                resource: light_occluders.clone(),
            },
            BindGroupEntry {
                binding:  2,
                resource: BindingResource::TextureView(&sdf_view_image.texture_view),
            },
        ],
    );

    let ss_probe_bind_group = render_device.create_bind_group(
        "gi_ss_probe_bind_group",
        &pipeline.ss_probe_bind_group_layout,
        &[
            BindGroupEntry {
                binding:  0,
                resource: camera_params.clone(),
            },
            BindGroupEntry {
                binding:  1,
                resource: gi_state.clone(),
            },
            BindGroupEntry {
                binding:  2,
                resource: probes.clone(),
            },
            BindGroupEntry {
                binding:  3,
                resource: skylight_masks.clone(),
            },
            BindGroupEntry {
                binding:  4,
                resource: light_sources.clone(),
            },
            BindGroupEntry {
                binding:  5,
                resource: BindingResource::TextureView(&sdf_view_image.texture_view),
            },
            BindGroupEntry {
                binding:  6,
                resource: BindingResource::Sampler(&sdf_view_image.sampler),
            },
            BindGroupEntry {
                binding:  7,
                resource: BindingResource::TextureView(&ss_probe_image.texture_view),
            },
        ],
    );

    let ss_bounce_bind_group = render_device.create_bind_group(
        "gi_bounce_bind_group",
        &pipeline.ss_bounce_bind_group_layout,
        &[
            BindGroupEntry {
                binding:  0,
                resource: camera_params.clone(),
            },
            BindGroupEntry {
                binding:  1,
                resource: gi_state.clone(),
            },
            BindGroupEntry {
                binding:  2,
                resource: BindingResource::TextureView(&sdf_view_image.texture_view),
            },
            BindGroupEntry {
                binding:  3,
                resource: BindingResource::Sampler(&sdf_view_image.sampler),
            },
            BindGroupEntry {
                binding:  4,
                resource: BindingResource::TextureView(&ss_probe_image.texture_view),
            },
            BindGroupEntry {
                binding:  5,
                resource: BindingResource::TextureView(&ss_bounce_image.texture_view),
            },
        ],
    );

    let ss_blend_bind_group = render_device.create_bind_group(
        "gi_blend_bind_group",
        &pipeline.ss_blend_bind_group_layout,
        &[
            BindGroupEntry {
                binding:  0,
                resource: camera_params.clone(),
            },
            BindGroupEntry {
                binding:  1,
                resource: gi_state.clone(),
            },
            BindGroupEntry {
                binding:  2,
                resource: probes.clone(),
            },
            BindGroupEntry {
                binding:  3,
                resource: BindingResource::TextureView(&sdf_view_image.texture_view),
            },
            BindGroupEntry {
                binding:  4,
                resource: BindingResource::Sampler(&sdf_view_image.sampler),
            },
            BindGroupEntry {
                binding:  5,
                resource: BindingResource::TextureView(&ss_bounce_image.texture_view),
            },
            BindGroupEntry {
                binding:  6,
                resource: BindingResource::TextureView(&ss_blend_image.texture_view),
            },
        ],
    );

    let ss_filter_bind_group = render_device.create_bind_group(
        "ss_filter_bind_group",
        &pipeline.ss_filter_bind_group_layout,
        &[
            BindGroupEntry {
                binding:  0,
                resource: camera_params.clone(),
            },
            BindGroupEntry {
                binding:  1,
                resource: gi_state.clone(),
            },
            BindGroupEntry {
                binding:  2,
                resource: probes.clone(),
            },
            BindGroupEntry {
                binding:  3,
                resource: BindingResource::TextureView(&sdf_view_image.texture_view),
            },
            BindGroupEntry {
                binding:  4,
                resource: BindingResource::Sampler(&sdf_view_image.sampler),
            },
            BindGroupEntry {
                binding:  5,
                resource: BindingResource::TextureView(&ss_blend_image.texture_view),
            },
            BindGroupEntry {
                binding:  6,
                resource: BindingResource::TextureView(&ss_filter_image.texture_view),
            },
            BindGroupEntry {
                binding:  7,
                resource: BindingResource::TextureView(&ss_pose_image.texture_view),
            },
        ],
    );

    log::info!("Successfully created all GI pipeline bind groups");
    commands.insert_resource(LightPassPipelineBindGroups {
        sdf_bind_group,
        ss_probe_bind_group,
        ss_bounce_bind_group,
        ss_blend_bind_group,
        ss_filter_bind_group,
    });
}

impl FromWorld for LightPassPipeline
{
    fn from_world(world: &mut World) -> Self
    {
        let render_device = world.resource::<RenderDevice>();

        let sdf_bind_group_layout = render_device.create_bind_group_layout(
            "sdf_bind_group_layout",
            &[
                // Camera.
                BindGroupLayoutEntry {
                    binding:    0,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuCameraParams::min_size()),
                    },
                    count:      None,
                },
                // Light occluders.
                BindGroupLayoutEntry {
                    binding:    1,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuLightOccluderBuffer::min_size()),
                    },
                    count:      None,
                },
                // SDF texture.
                BindGroupLayoutEntry {
                    binding:    2,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::ReadWrite,
                        format:         SDF_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
            ],
        );

        let ss_probe_bind_group_layout = render_device.create_bind_group_layout(
            "ss_probe_bind_group_layout",
            &[
                // Camera.
                BindGroupLayoutEntry {
                    binding:    0,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuCameraParams::min_size()),
                    },
                    count:      None,
                },
                // GI State.
                BindGroupLayoutEntry {
                    binding:    1,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuLightPassParams::min_size()),
                    },
                    count:      None,
                },
                // Probes.
                BindGroupLayoutEntry {
                    binding:    2,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuProbeDataBuffer::min_size()),
                    },
                    count:      None,
                },
                // SkylightMasks.
                BindGroupLayoutEntry {
                    binding:    3,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuSkylightMaskBuffer::min_size()),
                    },
                    count:      None,
                },
                // Light sources.
                BindGroupLayoutEntry {
                    binding:    4,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuLightSourceBuffer::min_size()),
                    },
                    count:      None,
                },
                // SDF.
                BindGroupLayoutEntry {
                    binding:    5,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count:      None,
                },
                // SDF Sampler.
                BindGroupLayoutEntry {
                    binding:    6,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Sampler(SamplerBindingType::Filtering),
                    count:      None,
                },
                // SS Probe.
                BindGroupLayoutEntry {
                    binding:    7,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::WriteOnly,
                        format:         SS_PROBE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
            ],
        );

        let ss_bounce_bind_group_layout = render_device.create_bind_group_layout(
            "ss_bounce_bind_group_layout",
            &[
                // Camera.
                BindGroupLayoutEntry {
                    binding:    0,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuCameraParams::min_size()),
                    },
                    count:      None,
                },
                // GI State.
                BindGroupLayoutEntry {
                    binding:    1,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuLightPassParams::min_size()),
                    },
                    count:      None,
                },
                // SDF.
                BindGroupLayoutEntry {
                    binding:    2,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count:      None,
                },
                // SDF Sampler.
                BindGroupLayoutEntry {
                    binding:    3,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Sampler(SamplerBindingType::Filtering),
                    count:      None,
                },
                // SS Probe.
                BindGroupLayoutEntry {
                    binding:    4,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::ReadOnly,
                        format:         SS_PROBE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
                // SS Bounce.
                BindGroupLayoutEntry {
                    binding:    5,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::WriteOnly,
                        format:         SS_BOUNCE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
            ],
        );

        let ss_blend_bind_group_layout = render_device.create_bind_group_layout(
            "ss_blend_bind_group_layout",
            &[
                // Camera.
                BindGroupLayoutEntry {
                    binding:    0,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuCameraParams::min_size()),
                    },
                    count:      None,
                },
                // GI State.
                BindGroupLayoutEntry {
                    binding:    1,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuLightPassParams::min_size()),
                    },
                    count:      None,
                },
                // Probes.
                BindGroupLayoutEntry {
                    binding:    2,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuProbeDataBuffer::min_size()),
                    },
                    count:      None,
                },
                // SDF.
                BindGroupLayoutEntry {
                    binding:    3,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count:      None,
                },
                // SDF Sampler.
                BindGroupLayoutEntry {
                    binding:    4,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Sampler(SamplerBindingType::Filtering),
                    count:      None,
                },
                // SS Bounces.
                BindGroupLayoutEntry {
                    binding:    5,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::ReadOnly,
                        format:         SS_BOUNCE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
                // SS Blend.
                BindGroupLayoutEntry {
                    binding:    6,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::WriteOnly,
                        format:         SS_BLEND_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
            ],
        );

        let ss_filter_bind_group_layout = render_device.create_bind_group_layout(
            "ss_filter_bind_group_layout",
            &[
                // Camera.
                BindGroupLayoutEntry {
                    binding:    0,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuCameraParams::min_size()),
                    },
                    count:      None,
                },
                // GI State.
                BindGroupLayoutEntry {
                    binding:    1,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuLightPassParams::min_size()),
                    },
                    count:      None,
                },
                // Probes.
                BindGroupLayoutEntry {
                    binding:    2,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Buffer {
                        ty:                 BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   Some(GpuProbeDataBuffer::min_size()),
                    },
                    count:      None,
                },
                // SDF.
                BindGroupLayoutEntry {
                    binding:    3,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Texture {
                        sample_type:    TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count:      None,
                },
                // SDF Sampler.
                BindGroupLayoutEntry {
                    binding:    4,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::Sampler(SamplerBindingType::Filtering),
                    count:      None,
                },
                // SS Blend.
                BindGroupLayoutEntry {
                    binding:    5,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::ReadOnly,
                        format:         SS_BLEND_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
                // SS Filter.
                BindGroupLayoutEntry {
                    binding:    6,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::WriteOnly,
                        format:         SS_FILTER_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
                // SS pose.
                BindGroupLayoutEntry {
                    binding:    7,
                    visibility: ShaderStages::COMPUTE,
                    ty:         BindingType::StorageTexture {
                        access:         StorageTextureAccess::WriteOnly,
                        format:         SS_POSE_TARGET_FORMAT,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count:      None,
                },
            ],
        );

        let (shader_sdf, gi_ss_probe, gi_ss_bounce, gi_ss_blend, gi_ss_filter) = {
            let assets_server = world.resource::<AssetServer>();
            (
                load_embedded_shader(assets_server, "gi_sdf.wgsl"),
                load_embedded_shader(assets_server, "gi_ss_probe.wgsl"),
                load_embedded_shader(assets_server, "gi_ss_bounce.wgsl"),
                load_embedded_shader(assets_server, "gi_ss_blend.wgsl"),
                load_embedded_shader(assets_server, "gi_ss_filter.wgsl"),
            )
        };

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let sdf_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label:                            Some("gi_sdf_pipeline".into()),
            layout:                           vec![sdf_bind_group_layout.clone()],
            shader:                           shader_sdf,
            shader_defs:                      vec![],
            entry_point:                      Some(SDF_PIPELINE_ENTRY.into()),
            push_constant_ranges:             vec![],
            zero_initialize_workgroup_memory: false,
        });

        let ss_probe_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label:                            Some("gi_ss_probe_pipeline".into()),
            layout:                           vec![ss_probe_bind_group_layout.clone()],
            shader:                           gi_ss_probe,
            shader_defs:                      vec![],
            entry_point:                      Some(SS_PROBE_PIPELINE_ENTRY.into()),
            push_constant_ranges:             vec![],
            zero_initialize_workgroup_memory: false,
        });

        let ss_bounce_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label:                            Some("gi_ss_bounce_pipeline".into()),
            layout:                           vec![ss_bounce_bind_group_layout.clone()],
            shader:                           gi_ss_bounce,
            shader_defs:                      vec![],
            entry_point:                      Some(SS_BOUNCE_PIPELINE_ENTRY.into()),
            push_constant_ranges:             vec![],
            zero_initialize_workgroup_memory: false,
        });

        let ss_blend_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label:                            Some("gi_blend_pipeline".into()),
            layout:                           vec![ss_blend_bind_group_layout.clone()],
            shader:                           gi_ss_blend,
            shader_defs:                      vec![],
            entry_point:                      Some(SS_BLEND_PIPELINE_ENTRY.into()),
            push_constant_ranges:             vec![],
            zero_initialize_workgroup_memory: false,
        });

        let ss_filter_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label:                            Some("gi_filer_pipeline".into()),
            layout:                           vec![ss_filter_bind_group_layout.clone()],
            shader:                           gi_ss_filter,
            shader_defs:                      vec![],
            entry_point:                      Some(SS_FILTER_PIPELINE_ENTRY.into()),
            push_constant_ranges:             vec![],
            zero_initialize_workgroup_memory: false,
        });

        LightPassPipeline {
            //
            sdf_bind_group_layout,
            sdf_pipeline,
            //
            ss_probe_bind_group_layout,
            ss_probe_pipeline,
            //
            ss_bounce_bind_group_layout,
            ss_bounce_pipeline,
            //
            ss_blend_bind_group_layout,
            ss_blend_pipeline,
            //
            ss_filter_bind_group_layout,
            ss_filter_pipeline,
        }
    }
}
