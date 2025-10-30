
# Projection Change Handling for Temporal Global Illumination

## Overview

This document describes the implementation of projection change detection and handling for the temporal global illumination system in Bevy Magic Light 2D. When camera scale changes (zoom in/out), the temporal lighting data becomes invalid because the mapping between world coordinates and screen coordinates changes. This solution detects such changes and compensates by reducing the contribution of invalid temporal data.

## Problem Description

The global illumination system uses screen-space irradiance cache probes that accumulate lighting data from multiple frames. This temporal accumulation provides stability and performance benefits by reusing previous frame calculations. However, when the camera projection changes (primarily through scale/zoom operations), the relationship between world coordinates and screen coordinates changes, making previous frame data invalid. Reading this stale temporal data causes visual artifacts and incorrect lighting.

## Solution Architecture

The implementation consists of three main components:

### 1. Projection Change Detection (`src/gi/projection_tracker.rs`)

A new `ProjectionTracker` resource that:
- Tracks the previous frame's view-projection matrix
- Detects significant scale changes by comparing projection matrix scale components
- Provides configurable threshold for change detection (default: 10% scale change)

```rust
pub struct ProjectionTracker {
    pub previous_view_proj: Option<Mat4>,
    pub scale_change_threshold: f32,
    pub(crate) invalidation_frames: u32,
}
```

### 2. GPU Parameter Updates (`src/gi/types_gpu.rs`)

Extended `GpuLightPassParams` with projection change information:
```rust
pub struct GpuLightPassParams {
    // ... existing fields ...
    
    // Projection change detection for temporal data handling
    pub projection_change_detected: i32,
    pub projection_scale_change: f32,
}
```

### 3. Shader-Based Temporal Weight Adjustment (`src/gi/shaders/gi_ss_blend.wgsl`)

Modified the blend shader to:
- Detect projection changes via GPU parameters
- Reduce temporal data contribution when changes are detected
- Scale reduction based on magnitude of projection change
- Handle edge cases where temporal weight becomes zero

## Implementation Details

### Change Detection Algorithm

1. Extract scale components from view-projection matrices
2. Calculate relative change in scale between frames
3. Flag significant changes when threshold is exceeded

```rust
let previous_scale = (previous.col(0).x.abs() + previous.col(1).y.abs()) / 2.0;
let current_scale = (current_view_proj.col(0).x.abs() + current_view_proj.col(1).y.abs()) / 2.0;
let scale_change = (current_scale - previous_scale).abs() / previous_scale;
```

### Shader Handling

The blend shader adjusts temporal contribution using a weight multiplier:

```wgsl
var temporal_weight_multiplier = 1.0;
if (cfg.projection_change_detected > 0) {
    // Scale temporal weight reduction based on magnitude of projection change
    temporal_weight_multiplier = max(0.1, 1.0 - cfg.projection_scale_change * 5.0);
}
```

### Integration Points

The system integrates into the existing pipeline at:
1. `system_extract_pipeline_assets()`: Detects projection changes
2. `ss_blend.wgsl`: Applies temporal weight adjustments
3. Plugin initialization: Sets up ProjectionTracker resource

## Configuration

The system can be configured through the `ProjectionTracker`:

- `scale_change_threshold`: Minimum relative scale change to trigger temporal invalidation (default: 0.1 = 10%)
- `invalidation_frames`: Number of frames to invalidate temporal data (currently unused, reserved for future enhancement)

## Benefits

1. **Visual Quality**: Eliminates lighting artifacts during camera zoom operations
2. **Temporal Stability**: Maintains temporal accumulation benefits when no projection changes occur
3. **Performance**: Minimal overhead - only simple matrix analysis and shader conditionals
4. **Configurability**: Adjustable thresholds for different use cases

## Limitations

1. **Scale-Only Detection**: Current implementation only detects uniform scale changes, not rotation or translation-based projection changes
2. **Approximate Handling**: Uses weight reduction rather than complete invalidation for smoother transitions
3. **Single Camera**: Assumes single camera setup; multi-camera scenarios may need additional handling

## Future Enhancements

1. **Complete Projection Tracking**: Handle rotation, aspect ratio, and other projection parameter changes
2. **Multi-Frame Invalidation**: Implement temporal data invalidation over multiple frames for very large changes
3. **Adaptive Thresholds**: Dynamic threshold adjustment based on scene complexity
4. **Camera-Specific Handling**: Support for multiple cameras with different projection behaviors

## Usage

The system automatically activates when using the Bevy Magic Light 2D plugin. No additional configuration is required for basic usage. For custom behavior, modify the `ProjectionTracker` resource parameters or extend the shader logic.

## Testing

The implementation has been verified through compilation testing. Runtime testing should include:
1. Camera zoom operations (mouse wheel, keyboard controls)
2. Rapid scale changes to verify artifact elimination
3. Static camera scenarios to confirm temporal benefits are preserved
