
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
            custom_size: Some(Vec2::new(2.0, 2.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)).with_scale(Vec3::splat(2.0)),
        Visibility::Visible,
    ));
}
