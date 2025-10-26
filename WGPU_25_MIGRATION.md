
# wgpu 25 Migration Guide for Bevy Magic Light 2D

This document outlines the changes made to migrate the Bevy Magic Light 2D project to wgpu 25 with Bevy 0.17.2.

## Background

wgpu 25 introduces breaking changes in bind group layout requirements:
- Dynamic offsets and uniforms can no longer be used in the same bind group as binding arrays
- New bind group numbering scheme:
  - `@group(0)` = view binding resources
  - `@group(1)` = view resources requiring binding arrays  
  - `@group(2)` = mesh binding resources
  - `@group(3)` = material binding resources

## Changes Made

### 1. Post-Processing Shader Bind Group Update

**File**: `src/gi/shaders/gi_post_processing.wgsl`

**Before**:
```wgsl
@group(2) @binding(0) var in_floor_texture:              texture_2d<f32>;
@group(2) @binding(1) var in_floor_sampler:              sampler;
// ... 8 total texture/sampler bindings using @group(2)
```

**After**:
```wgsl
@group(MATERIAL_BIND_GROUP) @binding(0) var in_floor_texture:              texture_2d<f32>;
@group(MATERIAL_BIND_GROUP) @binding(1) var in_floor_sampler:              sampler;
// ... 8 total texture/sampler bindings using MATERIAL_BIND_GROUP
```

### 2. Material Specialization Update

**File**: `src/gi/compositing.rs`

Added `MATERIAL_BIND_GROUP` shader definition to the post-processing material specialization:

```rust
shader_defs.push(ShaderDefVal::UInt(
    "MATERIAL_BIND_GROUP".to_string(),
    3u32,
));
```

### 3. Why This Approach

The post-processing shader is used by `PostProcessingMaterial` which implements `Material2d`. According to the new wgpu 25 layout, materials should use the material bind group. Using the `MATERIAL_BIND_GROUP` shader definition ensures:

1. **Future Compatibility**: If Bevy changes bind group numbering again, only the shader definition value needs to change
2. **Clear Intent**: Makes it explicit that this shader uses the material bind group
3. **Consistency**: Aligns with Bevy's recommended migration approach

## Migration Summary

- ✅ Updated `gi_post_processing.wgsl` to use `@group(MATERIAL_BIND_GROUP)` instead of `@group(2)`
- ✅ Added `MATERIAL_BIND_GROUP = 3` shader definition in material specialization
- ✅ Verified compilation with `cargo check`
- ✅ No float constants needed explicit typing (all constants already had proper types)

## Testing

The migration has been verified to compile successfully. The changes ensure compatibility with wgpu 25's stricter bind group layout requirements while maintaining the same functionality.

## Notes

- Other compute shaders in the project already use `@group(0)` correctly and don't require changes
- The migration follows Bevy's official guidelines for wgpu 25 compatibility
- No float constant declarations were found that needed explicit typing
