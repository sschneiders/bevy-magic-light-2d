
# Zoom Lighting Fix

## Problem Description

When zooming the camera in the bevy-magic-light-2d system, the illumination effect displayed visual artifacts. The issue was that the temporal accumulation system used lighting data from the previous frame that was still calculated with the old camera projection matrix.

### Root Cause

The temporal GI (Global Illumination) system accumulates lighting data across multiple frames to improve quality and reduce noise. However, it only compensated for camera translation motion, not changes in camera projection (zoom). During zoom operations:

1. The camera projection matrix changes
2. Previous frame lighting data becomes misaligned with the new view
3. Temporal accumulation blends old (incorrect) lighting with new lighting
4. This creates visible artifacts and instability during zoom

## Solution Implementation

### 1. Camera Projection Change Detection

Added zoom detection in `src/gi/pipeline_assets.rs`:

```rust
// Detect camera zoom/projection changes
let current_scale = projection.col(0).x; // For orthographic cameras, this represents the zoom level
let projection_change = if prev_view_proj.is_finite() {
    // Check for significant changes in projection matrix
    let view_proj_diff = (current_view_proj - *prev_view_proj).abs();
    let scale_diff = (current_scale - *prev_camera_scale).abs();
    
    // Calculate maximum absolute difference across all matrix elements
    let max_projection_diff = view_proj_diff.to_cols_array().into_iter().fold(0.0f32, |acc, x| acc.max(x));
    
    // Threshold for detecting zoom changes (adjustable)
    let zoom_threshold = 0.01;
    let projection_threshold = 0.1;
    
    // If projection or scale changed significantly, trigger temporal reset
    if scale_diff > zoom_threshold {
        1.0 // Reset temporal accumulation
    } else if max_projection_diff > projection_threshold {
        1.0 // Reset temporal accumulation
    } else {
        0.0 // Normal temporal accumulation
    }
} else {
    0.0 // First frame, no reset needed
};
```

### 2. GPU Data Structure Updates

Extended `GpuCameraParams` in `src/gi/types_gpu.rs` with temporal reset flag:

```rust
pub struct GpuCameraParams {
    pub screen_size:       Vec2,
    pub screen_size_inv:   Vec2,
    pub view_proj:         Mat4,
    pub inverse_view_proj: Mat4,
    pub sdf_scale:         Vec2,
    pub inv_sdf_scale:     Vec2,
    pub temporal_reset:    f32,  // NEW: Signals temporal reset during zoom
}
```

### 3. Shader-Level Temporal Reset

Modified the blending shader in `src/gi/shaders/gi_ss_blend.wgsl` to respect the reset signal:

```wgsl
// Apply temporal reset during zoom changes
let sample_weight = if (camera_params.temporal_reset > 0.5) {
    // During zoom changes, reduce weight of temporal samples
    r.weight * 0.1 // Significantly reduce temporal contribution
} else {
    r.weight
};
```

### 4. Camera Parameter System Updates

Updated the camera parameter extraction system to track previous frame state and detect zoom changes:

- Added `prev_view_proj` and `prev_camera_scale` tracking
- Implemented projection matrix change detection
- Added configurable thresholds for zoom sensitivity

## Usage

The zoom fix works automatically without requiring any configuration changes. However, you can test the functionality with the provided example:

```bash
cargo run --example zoom_test
```

### Example Controls

- **Mouse Wheel**: Zoom in/out
- **Keyboard**: `+/-` keys to zoom, `WASD` to move camera
- **Zoom Range**: 0.1x to 5.0x zoom level

## Technical Details

### Threshold Configuration

The system uses two thresholds to detect zoom changes:

1. **Zoom Threshold (`0.01`)**: Minimum scale change to trigger reset
2. **Projection Threshold (`0.1`)**: Maximum allowed projection matrix difference

These can be adjusted in `pipeline_assets.rs` based on specific needs:

```rust
let zoom_threshold = 0.01;        // Sensitive to small zoom changes
let projection_threshold = 0.1;   // Detects larger projection changes
```

### Temporal Reset Behavior

When a zoom change is detected:

1. `temporal_reset` flag is set to `1.0` in GPU parameters
2. Shader automatically reduces temporal sample weights to `0.1` (90% reduction)
3. This allows the lighting system to quickly re-converge with the new view
4. Normal temporal accumulation resumes on the next frame without zoom changes

### Performance Impact

- **Minimal overhead**: Only adds simple float comparisons during camera updates
- **GPU impact**: Negligible shader cost (one additional conditional)
- **Memory**: Adds 4 bytes to camera parameters buffer

## Benefits

1. **Artifact-free zooming**: Eliminates lighting artifacts during camera zoom operations
2. **Stable temporal accumulation**: Maintains lighting quality during normal movement
3. **Fast convergence**: Quickly adapts to new zoom levels
4. **Backward compatibility**: No breaking changes to existing code

## Testing

The `zoom_test.rs` example demonstrates the improvements with:

- Multiple colored lights in different positions
- Static occluders to show shadow stability
- Interactive camera controls for testing zoom scenarios
- Smooth camera transitions to verify temporal stability

## Future Improvements

Potential areas for enhancement:

1. **Adaptive thresholds**: Automatically adjust based on zoom speed
2. **Progressive reset**: Gradually reduce temporal weights instead of binary reset
3. **Motion-aware sampling**: Consider both translation and zoom in temporal reprojection
4. **Quality settings**: Allow users to adjust zoom sensitivity vs. quality trade-offs

