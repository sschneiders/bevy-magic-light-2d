

# Camera Viewer Feature

This document describes the camera viewer feature added to bevy-magic-light-2d, which provides an egui window to visualize the render targets of different camera layers.

## Overview

The camera viewer allows developers to:
- View render targets of individual camera layers (Floor, Walls, Objects)
- See the post-processing combined result
- View all layers side-by-side in a grid layout
- Understand how the lighting system affects each layer

## Components

### CameraViewerPlugin
The main plugin that adds the camera viewer functionality to your Bevy app.

### CameraType
Enum representing different camera layers:
- `Floor` - Floor layer render target
- `Walls` - Walls layer render target  
- `Objects` - Objects layer render target
- `PostProcessing` - Final combined result with lighting
- `Combined` - All layers shown side-by-side

### CameraViewerState
Resource that manages the camera viewer state:
- `selected_camera`: Currently selected camera type
- `show_window`: Whether the viewer window is visible

## Usage

### Basic Setup

1. Add the plugin to your app:
```rust
use bevy_magic_light_2d::prelude::*;

App::new()
    .add_plugins((
        DefaultPlugins,
        BevyMagicLight2DPlugin,
        EguiPlugin::default(),
        CameraViewerPlugin,
    ))
    .run();
```

2. Toggle the camera viewer window:
```rust
fn toggle_camera_viewer(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut viewer_state: ResMut<CameraViewerState>,
) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        viewer_state.show_window = !viewer_state.show_window;
    }
}
```

### Complete Example

See `examples/minimal_with_camera_viewer.rs` for a complete working example that demonstrates:
- Basic scene setup with lights and occluders
- Camera viewer integration
- Keyboard controls to toggle the viewer

## Features

### Render Target Visualization
- Shows placeholder rectangles representing each render target
- Displays target dimensions
- Provides clear labeling for each layer

### Camera Selection
- Dropdown menu to select different camera types
- Real-time switching between views
- Persistent selection across window toggles

### Combined View
- Grid layout showing all layers simultaneously
- Helps understand layer relationships
- Useful for debugging multi-layer scenes

## Integration

The camera viewer integrates with the existing bevy-magic-light-2d pipeline:

1. **CameraTargets**: Accesses floor, walls, and objects render targets
2. **Layer System**: Uses the existing layer system for organization
3. **Egui Integration**: Seamlessly integrates with bevy-inspector-egui

## API Reference

### CameraViewerPlugin
```rust
impl Plugin for CameraViewerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraViewerState>()
            .add_systems(Update, camera_viewer_ui_system);
    }
}
```

### CameraType
```rust
pub enum CameraType {
    Floor,
    Walls,
    Objects,
    PostProcessing,
    Combined,
}

impl CameraType {
    pub fn all_values() -> Vec<Self>;
    pub fn display_name(&self) -> &'static str;
    pub fn layers(&self) -> RenderLayers;
}
```

### CameraViewerState
```rust
pub struct CameraViewerState {
    pub selected_camera: CameraType,
    pub show_window: bool,
}

impl Default for CameraViewerState {
    fn default() -> Self {
        Self {
            selected_camera: CameraType::Floor,
            show_window: true,
        }
    }
}
```

## Controls

- **V Key**: Toggle camera viewer window on/off
- **Dropdown**: Select different camera/layers to view
- **Window Controls**: Minimize, close, or resize the viewer window

## Technical Notes

### Render Target Access
The viewer accesses render targets through the `CameraTargets` resource, which contains handles to:
- `floor_target`: Floor layer render target
- `walls_target`: Walls layer render target  
- `objects_target`: Objects layer render target

### Layer System Integration
Each `CameraType` maps to the existing layer system:
- `Floor` → `CAMERA_LAYER_FLOOR`
- `Walls` → `CAMERA_LAYER_WALLS`
- `Objects` → `CAMERA_LAYER_OBJECTS`
- `PostProcessing` → `CAMERA_LAYER_POST_PROCESSING`

### Egui Implementation
The viewer uses egui for the UI:
- Dropdown for camera selection
- Image display for render targets
- Grid layout for combined views
- Responsive resizing and positioning

## Future Improvements

Potential enhancements for the camera viewer:

1. **Real Texture Display**: Currently shows placeholders - could display actual render target content
2. **Zoom and Pan**: Allow zooming/panning of render target views
3. **Debug Overlays**: Show additional debug information about each layer
4. **Export Functionality**: Save render target views to disk
5. **Performance Metrics**: Display render time and memory usage per layer
6. **Comparison Mode**: Side-by-side comparison of different lighting settings

## Troubleshooting

### Common Issues

1. **Window Not Appearing**: Ensure `CameraViewerPlugin` is added and `show_window` is true
2. **Empty Views**: Check that `CameraTargets` resource is properly initialized
3. **Compilation Errors**: Make sure all required dependencies are included
4. **Post-Processing Shader Binding Issues**: The post-processing material now automatically updates when texture handles change, but you may need to enable debug logging to see the binding process

### Debug Tips

- Use `cargo run --example minimal_with_camera_viewer` to test basic functionality
- Check console output for any error messages
- Verify that render targets are properly created in your scene setup
- Enable debug logging to see texture binding information:
  ```rust
  use log::LevelFilter;
  
  // In your main setup:
  std::env::set_var("RUST_LOG", "debug");
  env_logger::init();
  ```

### Texture Binding Details

The post-processing system uses an `AsBindGroup` material that references:
- Floor layer render target (`@group(2) @binding(0)`)
- Walls layer render target (`@group(2) @binding(2)`)
- Objects layer render target (`@group(2) @binding(4)`)
- GI irradiance filter target (`@group(2) @binding(6)`)

The system now includes:
- Automatic material recreation when texture handles change
- Debug logging to track texture binding state
- Validation that all targets are properly initialized
- Proper cleanup during window resize events

If you're experiencing texture binding issues:
1. Check the console for debug messages about texture handle updates
2. Ensure GI targets are created before the post-processing material
3. Verify that all texture handles are properly initialized before rendering


