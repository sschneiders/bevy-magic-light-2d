use bevy::camera::visibility::RenderLayers;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MAX_CASCADES_PER_LIGHT, MAX_DIRECTIONAL_LIGHTS};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup,
    Extent3d,
    RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
    TextureDescriptor,
    TextureDimension,
    TextureFormat,
    TextureUsages,
};
use bevy::shader::{ShaderDefVal, ShaderRef};
use bevy::sprite_render::{Material2d, Material2dKey};

use crate::gi::constants::{POST_PROCESSING_MATERIAL, POST_PROCESSING_RECT};
use crate::gi::pipeline::GiTargetsWrapper;
use crate::gi::render_layer::CAMERA_LAYER_POST_PROCESSING;
use crate::gi::resource::ComputedTargetSizes;

#[derive(Component)]
pub struct PostProcessingQuad;

#[rustfmt::skip]
#[derive(AsBindGroup, Clone, TypePath, Asset)]
pub struct PostProcessingMaterial {
    #[texture(0)]
    #[sampler(1)]
    floor_image:       Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    walls_image:       Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    objects_image:     Handle<Image>,

    #[texture(6)]
    #[sampler(7)]
    irradiance_image:  Handle<Image>,
}

impl PostProcessingMaterial
{
    pub fn create(camera_targets: &CameraTargets, gi_targets_wrapper: &GiTargetsWrapper) -> Self
    {
        // Log texture handle information for debugging
        log::debug!("Creating PostProcessingMaterial with texture handles:");
        log::debug!("  floor_target: {:?}", camera_targets.floor_target);
        log::debug!("  walls_target: {:?}", camera_targets.walls_target);
        log::debug!("  objects_target: {:?}", camera_targets.objects_target);
        
        if let Some(ref targets) = gi_targets_wrapper.targets {
            log::debug!("  ss_filter_target: {:?}", targets.ss_filter_target);
        } else {
            log::error!("GI targets not initialized when creating PostProcessingMaterial!");
        }

        Self {
            floor_image:      camera_targets.floor_target.clone()
                .expect("Floor target must be initialized"),
            walls_image:      camera_targets.walls_target.clone()
                .expect("Walls target must be initialized"),
            objects_image:    camera_targets.objects_target.clone()
                .expect("Objects target must be initialized"),
            irradiance_image: gi_targets_wrapper
                .targets
                .as_ref()
                .expect("GI targets must be initialized")
                .ss_filter_target
                .clone(),
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraTargets
{
    pub floor_target:   Option<Handle<Image>>,
    pub walls_target:   Option<Handle<Image>>,
    pub objects_target: Option<Handle<Image>>,
}

impl CameraTargets
{
    pub fn update_handles(&mut self, images: &mut Assets<Image>, sizes: &ComputedTargetSizes)
    {
        let target_size = Extent3d {
            width: sizes.primary_target_usize.x,
            height: sizes.primary_target_usize.y,
            ..default()
        };

        let mut floor_image = Image {
            texture_descriptor: TextureDescriptor {
                label:           Some("target_floor"),
                size:            target_size,
                dimension:       TextureDimension::D2,
                format:          TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count:    1,
                usage:           TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats:    &[],
            },
            ..default()
        };
        let mut walls_image = Image {
            texture_descriptor: TextureDescriptor {
                label:           Some("target_walls"),
                size:            target_size,
                dimension:       TextureDimension::D2,
                format:          TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count:    1,
                usage:           TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats:    &[],
            },
            ..default()
        };

        let mut objects_image = Image {
            texture_descriptor: TextureDescriptor {
                label:           Some("target_objects"),
                size:            target_size,
                dimension:       TextureDimension::D2,
                format:          TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count:    1,
                usage:           TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats:    &[],
            },
            ..default()
        };

        // Fill images data with zeroes.
        floor_image.resize(target_size);
        walls_image.resize(target_size);
        objects_image.resize(target_size);

        if let Some(ref floor_target) = self.floor_target {
            images
                .insert(floor_target, floor_image)
                .expect("floor image handle updating should work everytime");
        } else {
            self.floor_target = Some(images.add(floor_image));
        }
        if let Some(ref walls_target) = self.walls_target {
            images
                .insert(walls_target, walls_image)
                .expect("walls image handle updating should work everytime");
        } else {
            self.walls_target = Some(images.add(walls_image));
        }
        if let Some(ref objects_target) = self.objects_target {
            images
                .insert(objects_target, objects_image)
                .expect("object image handle updating should work everytime");
        } else {
            self.objects_target = Some(images.add(objects_image));
        }
        
        // Validate that all targets are properly initialized
        if let (Some(floor), Some(walls), Some(objects)) = (&self.floor_target, &self.walls_target, &self.objects_target) {
            log::debug!("Camera targets updated successfully: floor={:?}, walls={:?}, objects={:?}", floor, walls, objects);
        } else {
            log::error!("Failed to initialize all camera targets!");
        }
    }
}

impl Material2d for PostProcessingMaterial
{
    fn fragment_shader() -> ShaderRef
    {
        "embedded://bevy_magic_light_2d/gi/shaders/gi_post_processing.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError>
    {
        let shader_defs = &mut descriptor
            .fragment
            .as_mut()
            .expect("Fragment shader empty")
            .shader_defs;
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_DIRECTIONAL_LIGHTS".to_string(),
            MAX_DIRECTIONAL_LIGHTS as u32,
        ));
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_CASCADES_PER_LIGHT".to_string(),
            MAX_CASCADES_PER_LIGHT as u32,
        ));
        Ok(())
    }
}

#[rustfmt::skip]
pub fn setup_post_processing_camera(
    mut commands:                  Commands,
    mut meshes:                    ResMut<Assets<Mesh>>,
    mut materials:                 ResMut<Assets<PostProcessingMaterial>>,
    mut images:                    ResMut<Assets<Image>>,
    mut camera_targets:            ResMut<CameraTargets>,

    target_sizes:                 Res<ComputedTargetSizes>,
    gi_targets_wrapper:           Res<GiTargetsWrapper>,
) {

    let quad =  Mesh::from(bevy::math::primitives::Rectangle::new(
        target_sizes.primary_target_size.x,
        target_sizes.primary_target_size.y,
    ));

    let _ = meshes.insert(POST_PROCESSING_RECT.id(), quad);

    camera_targets.update_handles(&mut images, &target_sizes);

    let material = PostProcessingMaterial::create(&camera_targets, &gi_targets_wrapper);
    let _ = materials.insert(POST_PROCESSING_MATERIAL.id(), material);

    // This specifies the layer used for the post processing camera, which
    // will be attached to the post processing camera and 2d quad.
    let layer = RenderLayers::layer(CAMERA_LAYER_POST_PROCESSING);

    commands.spawn((
        PostProcessingQuad,
        Mesh2d(POST_PROCESSING_RECT.clone()),
        MeshMaterial2d(POST_PROCESSING_MATERIAL.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
        layer.clone(),
    ));

    commands.spawn((
        Name::new("post_processing_camera"),
        Camera2d,
        Camera{
            order: 1,
            ..default()
        },
        Bloom {
            intensity: 0.1,
            ..default()
        },
        layer
    ))
    .insert((
        PostProcessingQuad,
        Mesh2d(POST_PROCESSING_RECT.clone()),
        MeshMaterial2d(POST_PROCESSING_MATERIAL.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
    ));
}
