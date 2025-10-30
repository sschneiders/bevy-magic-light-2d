use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// Tracks camera projection changes for temporal data invalidation
#[derive(Resource, Default, Clone, ExtractResource)]
pub struct ProjectionTracker {
    /// Previous frame's view-projection matrix
    pub previous_view_proj: Option<Mat4>,
    /// Scale threshold for detecting significant projection changes
    pub scale_change_threshold: f32,
    /// Number of frames to invalidate temporal data after projection change
    pub(crate) invalidation_frames: u32,
}

impl ProjectionTracker {
    /// Create a new projection tracker with default settings
    pub(crate) fn new() -> Self {
        Self {
            previous_view_proj: None,
            scale_change_threshold: 0.1, // 10% scale change threshold
            invalidation_frames: 3,      // Invalidate for 3 frames after change
        }
    }
    
    /// Detect if projection matrix has changed significantly
    pub fn detect_projection_change(&mut self, current_view_proj: Mat4) -> (bool, f32) {
        if let Some(previous) = self.previous_view_proj {
            // Extract scale from the matrices (assuming uniform scale for 2D)
            let previous_scale = (previous.col(0).x.abs() + previous.col(1).y.abs()) / 2.0;
            let current_scale = (current_view_proj.col(0).x.abs() + current_view_proj.col(1).y.abs()) / 2.0;
            
            if previous_scale > 0.0 {
                let scale_change = (current_scale - previous_scale).abs() / previous_scale;
                let significant_change = scale_change > self.scale_change_threshold;
                
                if significant_change {
                    log::debug!(
                        "Projection scale change detected: {:.3} -> {:.3} (Δ: {:.3})",
                        previous_scale,
                        current_scale,
                        scale_change
                    );
                }
                
                return (significant_change, scale_change);
            }
        }
        
        (false, 0.0)
    }
    
    /// Update the stored projection matrix
    pub fn update_projection(&mut self, view_proj: Mat4) {
        self.previous_view_proj = Some(view_proj);
    }
}
