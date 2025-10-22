

use bevy::prelude::*;




#[derive(Component)]
pub struct PostProcessingQuad;

pub fn setup_post_processing_camera(
    mut commands: Commands,
) {
    commands.spawn((
        PostProcessingQuad,
        Sprite {
            color: bevy::color::Color::WHITE,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)).with_scale(Vec3::splat(2.0)),
        Visibility::Visible,
    ));
}

#[derive(AsBindGroup, Asset, Clone, Debug, TypePath)]
pub struct PostProcessingMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub occlusion_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub gi_lighting_texture: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    pub gi_ambient_light_texture: Handle<Image>,

    #[texture(6)]
    #[sampler(7)]
    pub scene_texture: Handle<Image>,

    #[texture(8)]
    #[sampler(9)]
    pub scene_depth_texture: Handle<Image>,

    #[uniform(10)]
    pub ambient_strength: f32,
    #[uniform(11)]
    pub diffuse_strength: f32,
    #[uniform(12)]
    pub specular_strength: f32,
    #[uniform(13)]
    pub shininess: f32,
}

impl Material for PostProcessingMaterial {
    
}

#[derive(Component, Resource, Default)]
pub struct CameraTargets {
    pub image: Handle<Image>,
    pub size: UVec2,
}

#[derive(AsBindGroup, Asset, Clone, Debug, TypePath)]
pub struct CustomMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub color_texture: Handle<Image>,

    #[uniform(2)]
    pub emissive_strength: f32,
}

impl Material for CustomMaterial {
    
}

