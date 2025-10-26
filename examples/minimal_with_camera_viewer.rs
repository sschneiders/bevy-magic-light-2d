

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_magic_light_2d::prelude::*;

fn main()
{
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::srgba_u8(255, 255, 255, 0)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (512., 512.).into(),
                    title: "Bevy Magic Light 2D: Minimal + Camera Viewer".into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            BevyMagicLight2DPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            CameraViewerPlugin,
            ResourceInspectorPlugin::<BevyMagicLight2DSettings>::new(),
        ))
        .register_type::<LightOccluder2D>()
        .register_type::<OmniLightSource2D>()
        .register_type::<BevyMagicLight2DSettings>()
        .register_type::<LightPassParams>()
        .add_systems(Startup, setup.after(setup_post_processing_camera))
        .add_systems(Update, system_move_camera)
        .add_systems(Update, toggle_camera_viewer_simple)
        .run();
}

fn toggle_camera_viewer_simple(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut viewer_state: ResMut<CameraViewerState>,
) {
    // Press 'V' key to toggle camera viewer
    if keyboard.just_pressed(KeyCode::KeyV) {
        viewer_state.show_window = !viewer_state.show_window;
        println!("Camera viewer toggled: {}", viewer_state.show_window);
    }
}

fn setup(mut commands: Commands, camera_targets: Res<CameraTargets>)
{
    let mut occluders = vec![];
    let occluder_entity = commands
        .spawn((
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(40.0, 20.0),
            },
        ))
        .id();

    occluders.push(occluder_entity);

    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("occluders"))
        .add_children(&occluders);

    // Add lights.
    let mut lights = vec![];
    {
        let spawn_light = |cmd: &mut Commands,
                           x: f32,
                           y: f32,
                           name: &'static str,
                           light_source: OmniLightSource2D| {
            return cmd
                .spawn(Name::new(name))
                .insert(light_source)
                .insert((
                    Transform::from_translation(Vec3::new(x, y, 0.0)),
                    Visibility::default(),
                ))
                .id();
        };

        lights.push(spawn_light(
            &mut commands,
            -128.,
            -128.,
            "left",
            OmniLightSource2D {
                intensity: 1.0,
                color: Color::srgb_u8(255, 0, 0),
                falloff: Vec3::new(1.5, 10.0, 0.005),
                ..default()
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            128.,
            -128.,
            "right",
            OmniLightSource2D {
                intensity: 1.0,
                color: Color::srgb_u8(0, 0, 255),
                falloff: Vec3::new(1.5, 10.0, 0.005),
                ..default()
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            0.,
            128.,
            "top",
            OmniLightSource2D {
                intensity: 1.0,
                color: Color::srgb_u8(0, 255, 0),
                falloff: Vec3::new(1.5, 10.0, 0.005),
                ..default()
            },
        ));
    }
    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("lights"))
        .add_children(&lights);

    commands
        .spawn((
            Camera2d,
            Camera {
                hdr: true,
                target: RenderTarget::Image(camera_targets.floor_target.clone().into()),
                ..Default::default()
            },
            Name::new("main_camera"),
            FloorCamera,
        ))
        .insert(SpriteCamera);
}

fn system_move_camera(
    mut camera_target: Local<Vec3>,
    mut query_camera: Query<&mut Transform, With<SpriteCamera>>,

    keyboard: Res<ButtonInput<KeyCode>>,
)
{
    if let Ok(mut camera_transform) = query_camera.single_mut() {
        let speed = 10.0;

        if keyboard.pressed(KeyCode::KeyW) {
            camera_target.y += speed;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            camera_target.y -= speed;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            camera_target.x -= speed;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            camera_target.x += speed;
        }

        // Smooth camera.
        let blend_ratio = 0.18;
        let movement = (*camera_target - camera_transform.translation) * blend_ratio;
        camera_transform.translation.x += movement.x;
        camera_transform.translation.y += movement.y;
    }
}

