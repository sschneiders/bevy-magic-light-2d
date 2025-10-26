
# Bevy 0.17 Handle Migration

## Problem
The project was using UUID-based handles (formerly weak handles) from Bevy 0.16 that were causing black screen rendering issues in Bevy 0.17. The `uuid_handle!` macro creates handles that may not work correctly with the new asset system.

## Solution
Replaced all UUID handles with strong handles created via `assets.add()` and stored them in an `Option`-wrapped resource for proper lifecycle management and clearer initialization detection.

## Changes Made

### 1. New Resource Structure
**File: `src/gi/constants.rs`**
- Removed `POST_PROCESSING_RECT` and `POST_PROCESSING_MATERIAL` UUID handle constants
- Added `PostProcessingHandles` resource to store strong handles with `Option` for clear initialization state:
```rust
#[derive(Resource)]
pub struct PostProcessingHandles {
    pub rect_mesh: Option<Handle<Mesh>>,
    pub material: Option<Handle<PostProcessingMaterial>>,
}
```

### 2. Plugin Initialization
**File: `src/gi/mod.rs`**
- Added `.init_resource::<PostProcessingHandles>()` to plugin setup
- Updated import to use `PostProcessingHandles` instead of constants
- Updated `handle_window_resize` system to accept the new resource

### 3. Strong Handle Initialization

#### Window Resize Handler (`handle_window_resize`)
**File: `src/gi/mod.rs`**
- Checks if handles are uninitialized using `handle.is_none()`
- Creates strong handles via `assets.add()` when uninitialized
- Simplified logic by removing unnecessary `assets.insert()` calls

#### Post-Processing Camera Setup (`setup_post_processing_camera`)  
**File: `src/gi/compositing.rs`**
- Added `PostProcessingHandles` resource parameter
- Same initialization pattern as resize handler
- Extracts handles with proper `expect()` messages after initialization
- Updated all entity spawn calls to use handles from resource instead of constants

## Benefits

1. **Reliability**: Strong handles guarantee that assets remain loaded as long as needed
2. **Performance**: No UUID lookup overhead during asset access
3. **Bevy 0.17 Compatibility**: Properly integrated with the new asset system
4. **Resource Management**: Handles are properly managed through Bevy's resource system
5. **Clear Initialization**: `Option<Handle>` provides explicit initialization state
6. **Simplified Logic**: No need for unnecessary `assets.insert()` calls

## Testing
- All changes compile successfully with `cargo check`
- No breaking changes to external API
- Resource initialization is properly ordered in the plugin setup

## Migration Pattern
The approach follows Bevy 0.17 best practices:
- Use `assets.add()` to create strong handles
- Store handles in `Option`-wrapped resources for clear initialization management
- Use `handle.is_none()` to detect uninitialized state
- Extract handles with proper error handling using `expect()`

This pattern can be applied to other UUID handle migrations in the codebase if needed.
