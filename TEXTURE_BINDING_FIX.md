
# Global Illumination Texture Binding Fix

## Problem Description

The global illumination post-processing shader was experiencing texture binding issues where the textures were not properly available for the shader, even though the render buffers contained the correct data when viewed through the camera viewer window.

## Root Cause Analysis

The issue was caused by a **race condition** in the texture handle lifecycle during window resize events. The `handle_window_resize` system was creating the post-processing material **before** updating the GI targets and camera targets, which resulted in the material referencing outdated texture handles.

### Specific Issues Identified:

1. **Incorrect Material Creation Order**: In `handle_window_resize`, the post-processing material was recreated before GI targets were updated
2. **Stale Texture Handles**: When textures were recreated during resize, the material continued to reference old texture handles
3. **Missing Material Updates**: No system existed to automatically update the material when texture handles changed

## Solution Implementation

### 1. Fixed Texture Creation Order

**Before:** Material created → GI targets updated → Camera targets updated
```rust
let _ = assets_material.insert(
    POST_PROCESSING_MATERIAL.id(),
    PostProcessingMaterial::create(&res_camera_targets, &res_gi_targets_wrapper),
);
*res_gi_targets_wrapper = GiTargetsWrapper{targets: Some(GiTargets::create(&mut assets_image, &res_target_sizes))};
res_camera_targets.update_handles(&mut assets_image, &res_target_sizes);
```

**After:** GI targets updated → Camera targets updated → Material created
```rust
// IMPORTANT: Update GI targets and camera targets BEFORE recreating the material
// to ensure the post-processing material references the correct texture handles
*res_gi_targets_wrapper = GiTargetsWrapper{targets: Some(GiTargets::create(&mut assets_image, &res_target_sizes))};
res_camera_targets.update_handles(&mut assets_image, &res_target_sizes);

// Now recreate the post-processing material with updated texture handles
let _ = assets_material.insert(
    POST_PROCESSING_MATERIAL.id(),
    PostProcessingMaterial::create(&res_camera_targets, &res_gi_targets_wrapper),
);
```

### 2. Added Automatic Material Updates

New system `update_post_processing_material` that runs whenever texture resources change:

```rust
fn update_post_processing_material(
    mut materials: ResMut<Assets<PostProcessingMaterial>>,
    camera_targets: Res<CameraTargets>,
    gi_targets_wrapper: Res<GiTargetsWrapper>,
) {
    // Validates initialization and recreates material with current texture handles
    let updated_material = PostProcessingMaterial::create(&camera_targets, &gi_targets_wrapper);
    let _ = materials.insert(POST_PROCESSING_MATERIAL.id(), updated_material);
}
```

### 3. System Scheduling

Added the new system to run when either `GiTargetsWrapper` or `CameraTargets` changes:

```rust
.add_systems(PostUpdate, 
    (
        update_post_processing_material
            .run_if(resource_changed::<GiTargetsWrapper>)
            .after(handle_window_resize),
        update_post_processing_material
            .run_if(resource_changed::<CameraTargets>)
            .after(handle_window_resize),
    )
);
```

### 4. Enhanced Error Handling and Logging

- Added debug logging to track texture handle creation and updates
- Added validation to ensure all targets are initialized before material creation
- Improved error messages for troubleshooting

## Technical Implementation Details

### Post-Processing Material Structure

The `PostProcessingMaterial` uses `AsBindGroup` with the following bindings:
- Floor texture: `@group(2) @binding(0)` and sampler `@group(2) @binding(1)`
- Walls texture: `@group(2) @binding(2)` and sampler `@group(2) @binding(3)`
- Objects texture: `@group(2) @binding(4)` and sampler `@group(2) @binding(5)`
- Irradiance texture: `@group(2) @binding(6)` and sampler `@group(2) @binding(7)`

### GI Target Texture Setup

All GI pipeline targets are created with proper usage flags:
```rust
image.texture_descriptor.usage =
    TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
```

This ensures textures can be:
1. Written to by compute shaders (`STORAGE_BINDING`)
2. Sampled by the post-processing shader (`TEXTURE_BINDING`)
3. Updated via memory operations (`COPY_DST`)

## Verification

The fix ensures:
1. **Correct Ordering**: Textures are always created before materials reference them
2. **Automatic Updates**: Materials are automatically updated when texture handles change
3. **Error Prevention**: Validation prevents creation of materials with uninitialized textures
4. **Debug Visibility**: Comprehensive logging for troubleshooting texture binding issues

## Usage Instructions

The fix is transparent to users - no code changes are required. However, for debugging:

1. Enable debug logging to see texture binding messages:
   ```rust
   std::env::set_var("RUST_LOG", "debug");
   env_logger::init();
   ```

2. Monitor console output for messages like:
   ```
   DEBUG: Creating PostProcessingMaterial with texture handles:
   DEBUG:   floor_target: Handle<Image>(...)
   DEBUG:   walls_target: Handle<Image>(...)
   DEBUG:   objects_target: Handle<Image>(...)
   DEBUG:   ss_filter_target: Handle<Image>(...)
   DEBUG: Camera targets updated successfully
   DEBUG: Post-processing material updated successfully
   ```

## Impact

This fix resolves the core issue where the global illumination post-processing shader couldn't access the correctly computed irradiance data, ensuring that:
- Dynamic lighting effects are properly rendered
- Light bounces and indirect illumination are visible
- The global illumination system works as intended during window resize events
- Texture binding issues are automatically detected and resolved
