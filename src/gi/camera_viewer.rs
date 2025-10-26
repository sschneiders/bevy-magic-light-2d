
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use crate::gi::compositing::CameraTargets;
use crate::gi::render_layer::{
    CAMERA_LAYER_FLOOR, CAMERA_LAYER_OBJECTS, CAMERA_LAYER_WALLS, CAMERA_LAYER_POST_PROCESSING,
    ALL_LAYERS,
};

#[derive(Resource)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraType {
    Floor,
    Walls,
    Objects,
    PostProcessing,
    Combined,
}

impl CameraType {
    pub fn all_values() -> Vec<Self> {
        vec![
            Self::Floor,
            Self::Walls,
            Self::Objects,
            Self::PostProcessing,
            Self::Combined,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Floor => "Floor Layer",
            Self::Walls => "Walls Layer",
            Self::Objects => "Objects Layer",
            Self::PostProcessing => "Post Processing (Combined)",
            Self::Combined => "All Layers Combined",
        }
    }

    pub fn layers(&self) -> RenderLayers {
        match self {
            Self::Floor => RenderLayers::from_layers(CAMERA_LAYER_FLOOR),
            Self::Walls => RenderLayers::from_layers(CAMERA_LAYER_WALLS),
            Self::Objects => RenderLayers::from_layers(CAMERA_LAYER_OBJECTS),
            Self::PostProcessing => RenderLayers::from_layers(CAMERA_LAYER_POST_PROCESSING),
            Self::Combined => RenderLayers::from_layers(ALL_LAYERS),
        }
    }
}

pub struct CameraViewerPlugin;

impl Plugin for CameraViewerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraViewerState>()
            .add_systems(Update, camera_viewer_ui_system);
    }
}

fn camera_viewer_ui_system(
    mut egui_contexts: EguiContexts,
    camera_targets: Res<CameraTargets>,
    mut viewer_state: ResMut<CameraViewerState>,
    images: Res<Assets<Image>>,
) {
    if !viewer_state.show_window {
        // Show a toggle button in the main menu
        egui::Window::new("📷 Camera Viewer")
            .collapsible(false)
            .resizable(false)
            .show(egui_contexts.ctx_mut(), |ui| {
                if ui.button("Open Camera Viewer").clicked() {
                    viewer_state.show_window = true;
                }
            });
        return;
    }

    egui::Window::new("📷 Camera Viewer")
        .collapsible(true)
        .resizable(true)
        .default_size([400.0, 600.0])
        .show(egui_contexts.ctx_mut(), |ui| {
            ui.heading("Render Target Viewer");
            
            // Camera selection dropdown
            ui.horizontal(|ui| {
                ui.label("Camera:");
                let mut selected = viewer_state.selected_camera.clone();
                let mut changed = false;
                
                egui::ComboBox::from_label("")
                    .selected_text(selected.display_name())
                    .width(250.0)
                    .show_ui(ui, |ui| {
                        for camera_type in CameraType::all_values() {
                            if ui.selectable_label(
                                selected == camera_type,
                                camera_type.display_name()
                            ).clicked() {
                                selected = camera_type;
                                changed = true;
                            }
                        }
                    });
                
                if changed {
                    viewer_state.selected_camera = selected;
                }
            });

            ui.separator();

            // Display the selected camera's render target
            match viewer_state.selected_camera {
                CameraType::Floor => display_render_target(ui, &camera_targets.floor_target, &images, "Floor Layer"),
                CameraType::Walls => display_render_target(ui, &camera_targets.walls_target, &images, "Walls Layer"),
                CameraType::Objects => display_render_target(ui, &camera_targets.objects_target, &images, "Objects Layer"),
                CameraType::PostProcessing => {
                    ui.label("Post Processing View");
                    ui.label("(This is what you see in the main window)");
                    ui.label("The post processing combines all layers with lighting effects");
                },
                CameraType::Combined => {
                    ui.label("Combined View");
                    ui.label("Shows all render targets side by side:");
                    ui.separator();
                    
                    // Create a grid layout for all cameras
                    egui::Grid::new("camera_grid").num_columns(2).spacing([10.0, 10.0]).show(ui, |ui| {
                        display_render_target_in_grid(ui, &camera_targets.floor_target, &images, "Floor");
                        display_render_target_in_grid(ui, &camera_targets.walls_target, &images, "Walls");
                        display_render_target_in_grid(ui, &camera_targets.objects_target, &images, "Objects");
                        ui.end_row();
                    });
                },
            }

            ui.separator();

            // Instructions
            ui.collapsing("Instructions", |ui| {
                ui.label("• Select a camera to view its render target");
                ui.label("• The render targets show what each camera 'sees'");
                ui.label("• Floor, Walls, and Objects are separate layers");
                ui.label("• Post Processing combines all layers with lighting");
                ui.label("• Combined view shows all layers side by side");
            });

            // Close button
            if ui.button("Close").clicked() {
                viewer_state.show_window = false;
            }
        });
}

fn display_render_target(
    ui: &mut egui::Ui,
    target: &bevy::asset::Handle<Image>,
    images: &Assets<Image>,
    label: &str,
) {
    ui.label(label);
    
    if let Some(image) = images.get(target) {
        let size = image.size();
        
        // Create a simple placeholder visualization
        let available_size = ui.available_size();
        let aspect_ratio = size.x as f32 / size.y as f32;
        let display_height = available_size.y.min(400.0);
        let display_width = display_height * aspect_ratio;
        
        // Display a colored rectangle as placeholder
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(display_width, display_height),
            egui::Sense::hover(),
        );
        
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(4),
            egui::Color32::from_rgb(100, 100, 150),
        );
        
        // Add text overlay
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{} ({}×{})", label, size.x, size.y),
            egui::FontId::default(),
            egui::Color32::WHITE,
        );
        
        ui.label(format!("Size: {}×{}", size.x, size.y));
        ui.label("Render target visualization");
    } else {
        ui.label("Render target not available");
    }
}

fn display_render_target_in_grid(
    ui: &mut egui::Ui,
    target: &bevy::asset::Handle<Image>,
    images: &Assets<Image>,
    label: &str,
) {
    ui.label(label);
    
    if let Some(image) = images.get(target) {
        let size = image.size();
        
        // Create a small placeholder in the grid
        let display_size = 120.0;
        let aspect_ratio = size.x as f32 / size.y as f32;
        let display_height = display_size;
        let display_width = display_height * aspect_ratio;
        
        // Display a colored rectangle as placeholder
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(display_width, display_height),
            egui::Sense::hover(),
        );
        
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(4),
            egui::Color32::from_rgb(80, 80, 120),
        );
        
        // Add text overlay
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::default(),
            egui::Color32::WHITE,
        );
    } else {
        ui.label("Not available");
    }
}
