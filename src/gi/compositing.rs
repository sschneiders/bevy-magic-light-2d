use bevy::prelude::*;

use bevy::reflect::TypePath;
use bevy::mesh::MeshVertexBufferLayoutRef;
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
use bevy::prelude::*;
use bevy::asset::Handle;







use crate::gi::constants::POST_PROCESSING_MATERIAL;
use crate::gi::pipeline::GiTargetsWrapper;

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
        Self {
            floor_image:      camera_targets.floor_target.clone(),
            walls_image:      camera_targets.walls_target.clone(),
            objects_image:    camera_targets.objects_target.clone(),
            irradiance_image: gi_targets_wrapper
                .targets
                .as_ref()
                .expect("Targets must be initialized")
                .ss_filter_target
                .clone(),
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraTargets
{
    pub floor_target:   Handle<Image>,
    pub walls_target:   Handle<Image>,
    pub objects_target: Handle<Image>,
}

impl CameraTargets
{
    pub fn create(images: &mut Assets<Image>, sizes: &ComputedTargetSizes) -> Self
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

        let floor_image_handle: Handle<Image> = images.reserve_handle();
        let walls_image_handle: Handle<Image> = images.reserve_handle();
        let objects_image_handle: Handle<Image> = images.reserve_handle();

        images.insert(floor_image_handle.id(), floor_image);
        images.insert(walls_image_handle.id(), walls_image);
        images.insert(objects_image_handle.id(), objects_image);

        Self {
            floor_target:   floor_image_handle,
            walls_target:   walls_image_handle,
            objects_target: objects_image_handle,
        }
    }
}

impl Material2d for PostProcessingMaterial
{
    fn fragment_shader() -> ShaderRef
    {
        ShaderRef::Path("embedded://bevy_magic_light_2d/gi/shaders/gi_post_processing.wgsl".into())
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

    meshes.insert(POST_PROCESSING_RECT.id(), quad);

    *camera_targets = CameraTargets::create(&mut images, &target_sizes);

    let material = PostProcessingMaterial::create(&camera_targets, &gi_targets_wrapper);
    materials.insert(POST_PROCESSING_MATERIAL.id(), material);

    // Let layer be unused for now since we're switching to standard materials
    let _layer = 4u8;

    commands.spawn((
        PostProcessingQuad,
        Mesh2d(POST_PROCESSING_RECT.clone()),
        MeshMaterial2d(materials.add(PostProcessingMaterial {
            ambient_strength: 0.05,
            diffuse_strength: 1.0,
            specular_strength: 0.5,
            shininess: 32.0,
            occlusion_texture: computed_sizes.occlusion_target.clone(),
            gi_lighting_texture: computed_sizes.gi_lighting_target.clone(),
            gi_ambient_light_texture: computed_sizes.gi_ambient_light_target.clone(),
            scene_texture: computed_sizes.scene_target.clone(),
            scene_depth_texture: computed_sizes.scene_depth_target.clone(),
        })),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
        Visibility::Visible,
    ));

    commands.spawn((
        Name::new("post_processing_camera"),
        Camera2d, 
        Camera{
            order: 1,
    
            ..default()
        },
        Visibility::Visible,
    ));
}
