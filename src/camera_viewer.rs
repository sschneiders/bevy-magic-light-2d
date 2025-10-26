use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::{SpriteCamera, FloorCamera, WallsCamera, ObjectsCamera};

#[derive(Resource)]
pub struct CameraViewerState {
    pub selected_camera: CameraType,
    pub window_open: bool,
}

impl Default for CameraViewerState {
    fn default() -> Self {
        Self {
            selected_camera: CameraType::Floor,
            window_open: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraType {
    Floor,
    Walls,
    Objects,
    Sprite,
}

impl CameraType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CameraType::Floor => "Floor Camera",
            CameraType::Walls => "Walls Camera", 
            CameraType::Objects => "Objects Camera",
            CameraType::Sprite => "Sprite Camera",
        }
    }

    pub fn all() -> &'static [CameraType] {
        &[CameraType::Floor, CameraType::Walls, CameraType::Objects, CameraType::Sprite]
    }
}

pub fn setup_camera_viewer(mut commands: Commands) {
    commands.init_resource::<CameraViewerState>();
}

pub fn camera_viewer_window_system(
    mut contexts: EguiContexts,
    mut viewer_state: ResMut<CameraViewerState>,
    camera_targets: Res<crate::gi::compositing::CameraTargets>,
    cameras: Query<(&Camera, Option<&FloorCamera>, Option<&WallsCamera>, Option<&ObjectsCamera>, Option<&SpriteCamera>)>,
) {
    if !viewer_state.window_open {
        return;
    }

    let ctx = contexts.ctx_mut();
    let current_selected = viewer_state.selected_camera;
    
    if let Ok(ctx) = ctx {
        egui::Window::new("Camera Viewer")
            .open(&mut viewer_state.window_open)
            .resizable(true)
            .default_height(400.0)
            .show(ctx, |ui| {
            ui.heading("Camera View Selection");
            
            ui.horizontal(|ui| {
                ui.label("Select Camera:");
                
                // Camera selection combo box
                let mut selected_camera = current_selected;
                egui::ComboBox::from_label("")
                    .selected_text(selected_camera.as_str())
                    .show_ui(ui, |ui| {
                        for camera_type in CameraType::all() {
                            ui.selectable_value(
                                &mut selected_camera,
                                *camera_type,
                                camera_type.as_str(),
                            );
                        }
                    });
            });

            ui.separator();

            // Display the selected camera view using the cached selection
            let target_handle: Option<Handle<Image>> = match current_selected {
                CameraType::Floor => Some(camera_targets.floor_target.clone()),
                CameraType::Walls => Some(camera_targets.walls_target.clone()),
                CameraType::Objects => Some(camera_targets.objects_target.clone()),
                CameraType::Sprite => {
                    // For sprite camera, we need to find the camera and check its render target
                    cameras.iter()
                        .find_map(|(camera, _, _, _, sprite_cam)| {
                            sprite_cam.and_then(|_| {
                                if let bevy::camera::RenderTarget::Image(target) = &camera.target {
                                    // Extract the handle from ImageRenderTarget
                                    Some(target.handle.clone())
                                } else {
                                    None
                                }
                            })
                        })
                }
            };

            if let Some(_handle) = target_handle {
                ui.heading(format!("{} View", current_selected.as_str()));
                
                // Display the camera render target as an image
                let available_size = ui.available_size();
                let image_size = egui::Vec2::new(available_size.x.min(400.0), available_size.y.min(300.0));
                
                // Simple placeholder for now - we'll implement proper image display once compilation works
                ui.label(format!("Camera view would be displayed here (size: {:.0}x{:.0})", image_size.x, image_size.y));
                ui.separator();
                ui.label("Render target available for this camera");
            } else {
                ui.label("No render target available for selected camera");
            }
        });
    }

}
