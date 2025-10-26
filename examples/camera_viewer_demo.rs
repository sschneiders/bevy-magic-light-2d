
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy_inspector_egui::bevy_egui::{EguiPlugin, EguiContexts};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_magic_light_2d::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgba_u8(0, 0, 0, 0)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1024., 768.).into(),
                    title: "Bevy Magic Light 2D: Camera Viewer Demo".into(),
                    resizable: true,
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
        .add_systems(Update, (system_move_camera, toggle_camera_viewer))
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (2, 2),
                direct_light_contrib: 0.3,
                indirect_light_contrib: 0.7,
                ..default()
            },
            ..default()
        })
        .run();
}

fn setup(mut commands: Commands, camera_targets: Res<CameraTargets>) {
    // Create a more interesting scene with multiple occluders
    
    // Create walls/occluders
    let mut occluders = vec![];
    
    // Floor occluder
    let floor_occluder = commands
        .spawn((
            Transform::from_translation(Vec3::new(0., -200., 0.)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(300.0, 40.0),
            },
        ))
        .id();
    occluders.push(floor_occluder);
    
    // Top wall
    let top_wall = commands
        .spawn((
            Transform::from_translation(Vec3::new(0., 200., 0.)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(350.0, 30.0),
            },
        ))
        .id();
    occluders.push(top_wall);
    
    // Left wall
    let left_wall = commands
        .spawn((
            Transform::from_translation(Vec3::new(-250., 0., 0.)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(30.0, 200.0),
            },
        ))
        .id();
    occluders.push(left_wall);
    
    // Right wall
    let right_wall = commands
        .spawn((
            Transform::from_translation(Vec3::new(250., 0., 0.)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(30.0, 200.0),
            },
        ))
        .id();
    occluders.push(right_wall);
    
    // Center obstacle
    let center_obstacle = commands
        .spawn((
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(60.0, 60.0),
            },
        ))
        .id();
    occluders.push(center_obstacle);
    
    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("occluders"))
        .add_children(&occluders);

    // Add multiple lights for more interesting lighting effects
    let mut lights = vec![];
    
    // Red light in top-left
    lights.push(commands
        .spawn(Name::new("red_light"))
        .insert(OmniLightSource2D {
            intensity: 1.2,
            color: Color::srgb_u8(255, 50, 50),
            falloff: Vec3::new(1.0, 8.0, 0.01),
            ..default()
        })
        .insert((
            Transform::from_translation(Vec3::new(-150., 150., 0.0)),
            Visibility::default(),
        ))
        .id());

    // Blue light in top-right
    lights.push(commands
        .spawn(Name::new("blue_light"))
        .insert(OmniLightSource2D {
            intensity: 1.0,
            color: Color::srgb_u8(50, 100, 255),
            falloff: Vec3::new(1.2, 10.0, 0.008),
            ..default()
        })
        .insert((
            Transform::from_translation(Vec3::new(150., 150., 0.0)),
            Visibility::default(),
        ))
        .id());

    // Green light in bottom-left
    lights.push(commands
        .spawn(Name::new("green_light"))
        .insert(OmniLightSource2D {
            intensity: 0.8,
            color: Color::srgb_u8(50, 255, 100),
            falloff: Vec3::new(1.5, 12.0, 0.005),
            ..default()
        })
        .insert((
            Transform::from_translation(Vec3::new(-150., -150., 0.0)),
            Visibility::default(),
        ))
        .id());

    // Yellow light in bottom-right
    lights.push(commands
        .spawn(Name::new("yellow_light"))
        .insert(OmniLightSource2D {
            intensity: 0.9,
            color: Color::srgb_u8(255, 200, 50),
            falloff: Vec3::new(1.1, 9.0, 0.009),
            ..default()
        })
        .insert((
            Transform::from_translation(Vec3::new(150., -150., 0.0)),
            Visibility::default(),
        ))
        .id());

    // White center light (dimmer)
    lights.push(commands
        .spawn(Name::new("center_light"))
        .insert(OmniLightSource2D {
            intensity: 0.4,
            color: Color::srgb_u8(255, 255, 255),
            falloff: Vec3::new(2.0, 15.0, 0.003),
            ..default()
        })
        .insert((
            Transform::from_translation(Vec3::new(0., 0., 0.0)),
            Visibility::default(),
        ))
        .id());

    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("lights"))
        .add_children(&lights);

    // Main camera
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
) {
    if let Ok(mut camera_transform) = query_camera.single_mut() {
        let speed = 15.0;

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

        // Smooth camera movement
        let blend_ratio = 0.15;
        let movement = (*camera_target - camera_transform.translation) * blend_ratio;
        camera_transform.translation.x += movement.x;
        camera_transform.translation.y += movement.y;
    }
}

fn toggle_camera_viewer(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut egui_contexts: EguiContexts,
    mut viewer_state: ResMut<CameraViewerState>,
) {
    // Press 'V' key to toggle camera viewer
    if keyboard.just_pressed(KeyCode::KeyV) {
        viewer_state.show_window = !viewer_state.show_window;
    }
    
    // Show help text
    egui::Window::new("🎮 Controls")
        .collapsible(false)
        .resizable(false)
        .fixed_pos([10.0, 10.0])
        .show(egui_contexts.ctx_mut(), |ui| {
            ui.label("Camera Viewer Demo");
            ui.separator();
            ui.label("WASD - Move camera");
            ui.label("V - Toggle camera viewer");
            ui.separator();
            ui.label("The camera viewer lets you see the");
            ui.label("render targets for different layers:");
            ui.label("• Floor, Walls, Objects layers");
            ui.label("• Post-processing (final result)");
            ui.label("• Combined view of all layers");
        });
}
