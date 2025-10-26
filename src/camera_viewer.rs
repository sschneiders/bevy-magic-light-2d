use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
// Import removed as it's unused
use crate::{SpriteCamera, FloorCamera, WallsCamera, ObjectsCamera};

#[derive(Resource)]
pub struct CameraViewerState {
    pub selected_camera: CameraType,
    pub window_open: bool,
    loaded_texture_ids: std::collections::HashMap<CameraType, egui::TextureId>,
}

impl Default for CameraViewerState {
    fn default() -> Self {
        Self {
            selected_camera: CameraType::Floor,
            window_open: true,
            loaded_texture_ids: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    images: Res<Assets<Image>>,
    cameras: Query<(&Camera, Option<&FloorCamera>, Option<&WallsCamera>, Option<&ObjectsCamera>, Option<&SpriteCamera>)>,
) {
    if !viewer_state.window_open {
        return;
    }

    let ctx = contexts.ctx_mut();
    
    if let Ok(ctx) = ctx {
        // Copy current state to avoid borrow checker issues
        let current_selection = viewer_state.selected_camera;
        let mut window_open = viewer_state.window_open;
        let mut selected_camera = current_selection;
        
        egui::Window::new("Camera Viewer")
            .open(&mut window_open)
            .resizable(true)
            .default_height(400.0)
            .show(ctx, |ui| {
            ui.heading("Camera View Selection");
            
            ui.horizontal(|ui| {
                ui.label("Select Camera:");
                
                // Camera selection combo box
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

            // Display the selected camera view using the current selection
            let target_handle: Option<Handle<Image>> = match selected_camera {
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
                ui.heading(format!("{} View", selected_camera.as_str()));
                ui.label(format!("Handle: {:?}", _handle));
                
                // Display the camera render target as an image
                let available_size = ui.available_size();
                let image_size = egui::Vec2::new(available_size.x.min(400.0), available_size.y.min(300.0));
                
                // Display the actual render target using bevy_egui texture management
                if let Some(image) = images.get(&_handle) {
                    // Show basic image info
                    ui.label(format!("Image Size: {}x{}", 
                        image.texture_descriptor.size.width, 
                        image.texture_descriptor.size.height));
                    ui.label(format!("Format: {:?}", image.texture_descriptor.format));
                    ui.label(format!("Data available: {:?}", image.data.is_some()));
                    if let Some(data) = &image.data {
                        ui.label(format!("Data length: {:?}", data.len()));
                    } else {
                        ui.label("No data available");
                    }
                    ui.label(format!("Texture descriptor: {:?}", image.texture_descriptor));
                    
                    // Display the actual render target texture
                    ui.label("Render Target:");
                    
                    // Try to display the actual image data
                    if let Some(data) = &image.data {
                        if !data.is_empty() {
                            ui.label(format!("✓ Texture loaded | Size: {} bytes", data.len()));
                            
                            // Check if we can get the format right for egui display
                            match image.texture_descriptor.format {
                                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb |
                                bevy::render::render_resource::TextureFormat::Rgba8Unorm => {
                                    // Create egui ColorImage from Bevy Image data
                                    let size = [image.texture_descriptor.size.width as usize, 
                                               image.texture_descriptor.size.height as usize];
                                    
                                    // Convert the image data for egui - need to ensure proper format
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, data);
                                    
                                    // Load texture into egui and store the texture ID
                                    let texture_name = format!("camera_render_{:?}", selected_camera);
                                    let texture_handle = ui.ctx().load_texture(
                                        texture_name,
                                        color_image,
                                        egui::TextureOptions::default(),
                                    );
                                    
                                    // Store the texture ID for future use
                                    let texture_id = texture_handle.id();
                                    viewer_state.loaded_texture_ids.insert(selected_camera, texture_id);
                                    
                                    // Display the actual image using the correct texture ID
                                    let response = ui.image(egui::load::SizedTexture::new(
                                        texture_id,
                                        image_size
                                    ));
                                    
                                    // Add visual feedback and grid overlay to help distinguish content
                                    if response.hovered() {
                                        let painter = ui.painter();
                                        let rect = response.rect;
                                        
                                        // Draw semi-transparent grid to help see what's being displayed
                                        let grid_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, 30);
                                        let stroke = egui::Stroke::new(1.0, grid_color);
                                        
                                        // Vertical lines
                                        for i in 1..4 {
                                            let x = rect.min.x + (rect.width() / 4.0 * i as f32);
                                            painter.line_segment(
                                                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                                                stroke
                                            );
                                        }
                                        
                                        // Horizontal lines
                                        for i in 1..3 {
                                            let y = rect.min.y + (rect.height() / 3.0 * i as f32);
                                            painter.line_segment(
                                                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                                                stroke
                                            );
                                        }
                                        
                                        // Show center crosshair
                                        let center = rect.center();
                                        painter.line_segment(
                                            [egui::pos2(center.x - 10.0, center.y), egui::pos2(center.x + 10.0, center.y)],
                                            egui::Stroke::new(1.0, egui::Color32::RED)
                                        );
                                        painter.line_segment(
                                            [egui::pos2(center.x, center.y - 10.0), egui::pos2(center.x, center.y + 10.0)],
                                            egui::Stroke::new(1.0, egui::Color32::RED)
                                        );
                                    }
                                }
                                _ => {
                                    ui.label(format!("Unsupported format: {:?}", image.texture_descriptor.format));
                                    // Show fallback visual
                                    let rect = egui::Rect::from_min_size(ui.cursor().min, image_size);
                                    let camera_color = match selected_camera {
                                        CameraType::Floor => egui::Color32::from_rgb(80, 140, 80),
                                        CameraType::Walls => egui::Color32::from_rgb(140, 80, 80),
                                        CameraType::Objects => egui::Color32::from_rgb(80, 80, 140),
                                        CameraType::Sprite => egui::Color32::from_rgb(140, 140, 80),
                                    };
                                    ui.painter().rect_filled(rect, 4.0, camera_color);
                                    ui.add_space(image_size.y);
                                }
                            }
                        } else {
                            ui.label("✗ Texture data is empty");
                            // Show empty texture placeholder
                            let rect = egui::Rect::from_min_size(ui.cursor().min, image_size);
                            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 40, 40));
                            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)), egui::StrokeKind::Inside);
                            ui.add_space(image_size.y);
                        }
                    } else {
                        ui.label("✗ No texture data available");
                        // Show placeholder for missing texture
                        let rect = egui::Rect::from_min_size(ui.cursor().min, image_size);
                        ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 40, 40));
                        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)), egui::StrokeKind::Inside);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{} Camera\n[No Render Target]", selected_camera.as_str()),
                            egui::FontId::default(),
                            egui::Color32::from_rgb(150, 150, 150),
                        );
                        ui.add_space(image_size.y);
                    }
                    ui.separator();
                    
                    // Additional controls for the camera view
                    ui.horizontal(|ui| {
                        if ui.button("Refresh").clicked() {
                            // Trigger texture refresh - to be implemented
                        }
                        if ui.button("Save View").clicked() {
                            // Save current camera view - to be implemented
                        }
                    });
                } else {
                    // Fallback to placeholder if image isn't loaded yet
                    let rect = egui::Rect::from_min_size(ui.cursor().min, image_size);
                    ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(50, 50, 50));
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{} Camera\nNo render target", selected_camera.as_str()),
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                    ui.add_space(image_size.y);
                }
                
                ui.separator();
                ui.label(format!("{} - Render target displayed", selected_camera.as_str()));
            } else {
                ui.label("No render target available for selected camera");
                
                // Debug: Show what we have access to
                ui.separator();
                ui.label("Debug Information:");
                ui.label(format!("Selected Camera: {:?}", selected_camera));
                ui.label("Available CameraTargets:");
                ui.label(format!("Floor target: {:?}", camera_targets.floor_target));
                ui.label(format!("Walls target: {:?}", camera_targets.walls_target));
                ui.label(format!("Objects target: {:?}", camera_targets.objects_target));
                
                // Check if we can access the images
                ui.label("Image Asset Access:");
                if let Some(_floor_image) = images.get(&camera_targets.floor_target) {
                    ui.label("✓ Floor image accessible");
                } else {
                    ui.label("✗ Floor image not accessible");
                }
                if let Some(_walls_image) = images.get(&camera_targets.walls_target) {
                    ui.label("✓ Walls image accessible");
                } else {
                    ui.label("✗ Walls image not accessible");
                }
                if let Some(_objects_image) = images.get(&camera_targets.objects_target) {
                    ui.label("✓ Objects image accessible");
                } else {
                    ui.label("✗ Objects image not accessible");
                }
            }
        });
        
        // Update the viewer state after window interactions
        if selected_camera != current_selection {
            viewer_state.selected_camera = selected_camera;
        }
        viewer_state.window_open = window_open;
    }

}
