
use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::camera::RenderTarget;
use bevy_inspector_egui::quick::*;
use bevy_magic_light_2d::prelude::*;

fn main()
{
    // Enable debug logging to track zoom detection
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    // Basic setup with zoom capability
    App::new()
        .insert_resource(ClearColor(Color::srgba_u8(20, 20, 30, 255)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1024u32, 768u32).into(),
                    title: "Bevy Magic Light 2D: Zoom Test Example".into(),
                    resizable: true,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            BevyMagicLight2DPlugin,
            ResourceInspectorPlugin::<BevyMagicLight2DSettings>::new(),
        ))
        .register_type::<BevyMagicLight2DSettings>()
        .register_type::<LightPassParams>()
        .add_systems(Startup, setup.after(setup_post_processing_camera))
        .add_systems(Update, system_zoom_camera)
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (3, 3),
                direct_light_contrib: 0.4,
                indirect_light_contrib: 0.6,
                indirect_rays_per_sample: 32,
                ..default()
            },
            ..default()
        })
        .run();
}

fn setup(mut commands: Commands, camera_targets: Res<CameraTargets>)
{
    let mut occluders = vec![];
    
    // Create static occluders
    let occluder_entities = [
        // Large walls
        commands.spawn((
            Transform::from_translation(Vec3::new(0.0, -200.0, 0.0)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(300.0, 20.0),
            },
        )).id(),
        
        commands.spawn((
            Transform::from_translation(Vec3::new(0.0, 200.0, 0.0)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(300.0, 20.0),
            },
        )).id(),
        
        commands.spawn((
            Transform::from_translation(Vec3::new(-250.0, 0.0, 0.0)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(20.0, 150.0),
            },
        )).id(),
        
        commands.spawn((
            Transform::from_translation(Vec3::new(250.0, 0.0, 0.0)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(20.0, 150.0),
            },
        )).id(),
        
        // Central obstacle
        commands.spawn((
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            Visibility::default(),
            LightOccluder2D {
                h_size: Vec2::new(60.0, 60.0),
            },
        )).id(),
    ];

    occluders.extend(occluder_entities);

    commands
        .spawn((Visibility::default(), Transform::default()))
        .insert(Name::new("occluders"))
        .add_children(&occluders);

    // Add lights with different colors and positions
    let mut lights = vec![];
    let light_configs = [
        (-200., -150., "top_left", Color::srgb_u8(255, 100, 100), 12.0),
        (200., -150., "top_right", Color::srgb_u8(100, 100, 255), 12.0),
        (0., 150., "bottom_center", Color::srgb_u8(100, 255, 100), 10.0),
        (-150., 50., "side_left", Color::srgb_u8(255, 255, 100), 8.0),
        (150., 50., "side_right", Color::srgb_u8(255, 100, 255), 8.0),
    ];

    for (x, y, name, color, intensity) in light_configs {
        lights.push(commands.spawn((
            Name::new(name),
            OmniLightSource2D {
                intensity,
                color,
                falloff: Vec3::new(2.0, 15.0, 0.01),
                jitter_intensity: 0.2,
                jitter_translation: 1.0,
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Visibility::default(),
        )).id());
    }

    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("lights"))
        .add_children(&lights);

    // Setup camera with zoom capability
    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Image(camera_targets.floor_target.clone().expect("Floor target must be initialized").into()),
            ..Default::default()
        },
        Name::new("main_camera"),
        FloorCamera,
        ZoomState {
            zoom: 1.0,
            target_zoom: 1.0,
            position: Vec3::ZERO,
            target_position: Vec3::ZERO,
        },
    ));
}

#[derive(Component)]
struct ZoomState {
    zoom: f32,
    target_zoom: f32,
    position: Vec3,
    target_position: Vec3,
}

fn system_zoom_camera(
    mut query_camera: Query<(&mut Transform, &mut ZoomState), With<FloorCamera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    _mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut zoom_state)) = query_camera.single_mut() {
        let delta = time.delta_secs();

        // Mouse wheel zoom
        let mut scroll_delta = 0.0;
        for event in mouse_wheel_events.read() {
            scroll_delta += event.y;
        }
        
        if scroll_delta != 0.0 {
            let zoom_speed = 0.1;
            zoom_state.target_zoom *= 1.0 + scroll_delta * zoom_speed;
            zoom_state.target_zoom = zoom_state.target_zoom.clamp(0.1, 5.0);
        }

        // Keyboard zoom controls
        let keyboard_zoom_speed = 2.0 * delta;
        if keyboard.pressed(KeyCode::Equal) {
            zoom_state.target_zoom += keyboard_zoom_speed;
        }
        if keyboard.pressed(KeyCode::Minus) {
            zoom_state.target_zoom -= keyboard_zoom_speed;
        }
        zoom_state.target_zoom = zoom_state.target_zoom.clamp(0.1, 5.0);

        // Camera movement with WASD
        let movement_speed = 200.0 * delta / zoom_state.zoom; // Adjust speed based on zoom
        let mut movement = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            movement.y += movement_speed;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement.y -= movement_speed;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            movement.x -= movement_speed;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement.x += movement_speed;
        }

        zoom_state.target_position += movement;

        // Smooth transitions for zoom and position
        let zoom_smoothness = 10.0;
        let position_smoothness = 8.0;

        zoom_state.zoom += (zoom_state.target_zoom - zoom_state.zoom) * zoom_smoothness * delta;
        
        let target_pos = zoom_state.target_position;
        let current_pos = zoom_state.position;
        zoom_state.position += (target_pos - current_pos) * position_smoothness * delta;

        // Apply transform changes
        transform.translation = zoom_state.position;
        transform.scale = Vec3::splat(zoom_state.zoom);
    }
}

// UI system removed to simplify the example
