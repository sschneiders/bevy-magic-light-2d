use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use log::info;

use crate::gi::compositing::CameraTargets;
use crate::gi::render_layer::{
    ALL_LAYERS,
    CAMERA_LAYER_FLOOR,
    CAMERA_LAYER_OBJECTS,
    CAMERA_LAYER_POST_PROCESSING,
    CAMERA_LAYER_WALLS,
};




#[derive(Resource)]
pub struct CameraViewerState
{
    pub selected_camera: CameraType,
    pub show_window:     bool,
}

impl Default for CameraViewerState
{
    fn default() -> Self
    {
        Self {
            selected_camera: CameraType::Floor,
            show_window:     false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraType
{
    Floor,
    Walls,
    Objects,
    PostProcessing,
    Combined,
}

impl CameraType
{
    pub fn all_values() -> Vec<Self>
    {
        vec![
            Self::Floor,
            Self::Walls,
            Self::Objects,
            Self::PostProcessing,
            Self::Combined,
        ]
    }

    pub fn display_name(&self) -> &'static str
    {
        match self {
            Self::Floor => "Floor Layer",
            Self::Walls => "Walls Layer",
            Self::Objects => "Objects Layer",
            Self::PostProcessing => "Post Processing (Combined)",
            Self::Combined => "All Layers Combined",
        }
    }

    pub fn layers(&self) -> RenderLayers
    {
        match self {
            Self::Floor => RenderLayers::layer(CAMERA_LAYER_FLOOR),
            Self::Walls => RenderLayers::layer(CAMERA_LAYER_WALLS),
            Self::Objects => RenderLayers::layer(CAMERA_LAYER_OBJECTS),
            Self::PostProcessing => RenderLayers::layer(CAMERA_LAYER_POST_PROCESSING),
            Self::Combined => RenderLayers::from_layers(ALL_LAYERS),
        }
    }
}

pub struct CameraViewerPlugin;

impl Plugin for CameraViewerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<CameraViewerState>()
            .add_systems(EguiPrimaryContextPass, camera_viewer_ui_system)
            .add_systems(Update, register_render_target_textures.run_if(resource_changed::<CameraTargets>));
    }
}

fn register_render_target_textures(
    camera_targets: Res<CameraTargets>,
    mut egui_user_textures: EguiContexts,
) {
    info!("Registering render target textures with egui...");

    egui_user_textures.add_image(bevy_egui::EguiTextureHandle::Strong(camera_targets.floor_target.clone().unwrap()));
    egui_user_textures.add_image(bevy_egui::EguiTextureHandle::Strong(
        camera_targets.walls_target.clone().unwrap(),
    ));
    egui_user_textures.add_image(bevy_egui::EguiTextureHandle::Strong(
        camera_targets.objects_target.clone().unwrap(),
    ));

    info!("Done Render target textures registered with egui!");
}

fn camera_viewer_ui_system(
    mut egui_contexts: EguiContexts,
    camera_targets: Res<CameraTargets>,
    mut viewer_state: ResMut<CameraViewerState>,
    images: Res<Assets<Image>>,
)
{
    // Check texture IDs before the window to avoid borrowing issues
    let floor_texture_id = egui_contexts.image_id(camera_targets.floor_target.as_ref().unwrap());
    let walls_texture_id = egui_contexts.image_id(camera_targets.walls_target.as_ref().unwrap());
    let objects_texture_id = egui_contexts.image_id(camera_targets.objects_target.as_ref().unwrap());

    let Ok(ctx) = egui_contexts.ctx_mut() else {
        return;
    };
    if !viewer_state.show_window {
        // Show a more prominent toggle button
        egui::Window::new("📷 Camera Viewer")
            .collapsible(false)
            .resizable(false)
            .default_pos([10.0, 50.0])
            .show(ctx, |ui| {
                ui.heading("Camera Viewer");
                ui.separator();
                ui.label("Press 'V' or click the button below to open the camera viewer");
                if ui.button("📷 Open Camera Viewer").clicked() {
                    viewer_state.show_window = true;
                }
                ui.separator();
                ui.label("The camera viewer lets you see:");
                ui.label("• Individual render targets (Floor, Walls, Objects)");
                ui.label("• Combined post-processing result");
                ui.label("• All layers side-by-side");
            });
        return;
    }

    egui::Window::new("📷 Camera Viewer")
        .collapsible(true)
        .resizable(true)
        .default_size([450.0, 650.0])
        .default_pos([50.0, 80.0])
        .show(ctx, |ui| {
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
                            if ui
                                .selectable_label(
                                    selected == camera_type,
                                    camera_type.display_name(),
                                )
                                .clicked()
                            {
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
                CameraType::Floor => display_render_target(
                    ui,
                    &camera_targets.floor_target,
                    &images,
                    "Floor Layer",
                    floor_texture_id,
                ),
                CameraType::Walls => display_render_target(
                    ui,
                    &camera_targets.walls_target,
                    &images,
                    "Walls Layer",
                    walls_texture_id,
                ),
                CameraType::Objects => display_render_target(
                    ui,
                    &camera_targets.objects_target,
                    &images,
                    "Objects Layer",
                    objects_texture_id,
                ),
                CameraType::PostProcessing => {
                    ui.label("Post Processing View");
                    ui.label("(This is what you see in the main window)");
                    ui.label("The post processing combines all layers with lighting effects");
                }
                CameraType::Combined => {
                    ui.label("Combined View");
                    ui.label("Shows all render targets side by side:");
                    ui.separator();

                    // Create a grid layout for all cameras
                    egui::Grid::new("camera_grid")
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            display_render_target_in_grid(
                                ui,
                                &camera_targets.floor_target,
                                &images,
                                "Floor",
                                floor_texture_id,
                            );
                            display_render_target_in_grid(
                                ui,
                                &camera_targets.walls_target,
                                &images,
                                "Walls",
                                walls_texture_id,
                            );
                            display_render_target_in_grid(
                                ui,
                                &camera_targets.objects_target,
                                &images,
                                "Objects",
                                objects_texture_id,
                            );
                            ui.end_row();
                        });
                }
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
    target: &Option<Handle<Image>>,
    images: &Assets<Image>,
    label: &str,
    texture_id: Option<egui::TextureId>,
)
{
    let Some(target) = target else {
        error!("display_render_target called with no target");
        return;
    };

    ui.label(label);

    if let Some(image) = images.get(target) {
        let size = image.size();

        // Create a simple placeholder visualization
        let available_size = ui.available_size();
        let aspect_ratio = size.x as f32 / size.y as f32;
        let display_height = available_size.y.min(400.0);
        let display_width = display_height * aspect_ratio;

        // Try to display the actual render target image using egui texture system
        match texture_id {
            Some(texture_id) => {
                // Successfully got egui texture ID, display the actual image
                ui.image(egui::load::SizedTexture::new(
                    texture_id,
                    egui::Vec2::new(display_width, display_height),
                ));

                // Show some debug info
                if let Some(data) = &image.data {
                    let data_size = data.len();
                    let pixel_count = data_size / 4;
                    ui.label(format!("✅ Render target: {} pixels", pixel_count));
                    ui.label(format!("Buffer size: {} bytes", data_size));
                } else {
                    ui.label("✅ Render target loaded (no CPU data access)");
                }
            }
            None => {
                // Fallback to visualization if texture not registered with egui
                ui.label("⚠️ Texture not registered with egui");

                if let Some(image) = images.get(target) {
                    if let Some(data) = &image.data {
                        let data_size = data.len();
                        let pixel_count = data_size / 4;
                        ui.label(format!("Has data: {} pixels", pixel_count));
                    } else {
                        ui.label("Has handle but no CPU data");
                    }

                    // Simple colored rectangle to indicate this render target exists
                    let (rect, _) = ui.allocate_exact_size(
                        egui::Vec2::new(display_width, display_height),
                        egui::Sense::hover(),
                    );

                    // Use different colors for different render targets
                    let color = match label {
                        "Floor Layer" => egui::Color32::from_rgb(100, 150, 100),
                        "Walls Layer" => egui::Color32::from_rgb(150, 100, 100),
                        "Objects Layer" => egui::Color32::from_rgb(100, 100, 150),
                        _ => egui::Color32::from_rgb(120, 120, 120),
                    };

                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(4), color);

                    // Add text overlay
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{} ({}×{})", label, size.x, size.y),
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                } else {
                    ui.label("❌ Render target not found");
                }
            }
        }

        ui.label(format!("Size: {}×{}", size.x, size.y));
    } else {
        ui.label("Render target not available");
    }
}

fn display_render_target_in_grid(
    ui: &mut egui::Ui,
    target: &Option<Handle<Image>>,
    images: &Assets<Image>,
    label: &str,
    texture_id: Option<egui::TextureId>,
)
{
    let Some(ref target) = target else {
        error!("display_render_target_in_grid no target");
        return;
    };
    ui.label(label);

    if let Some(image) = images.get(target) {
        let size = image.size();

        // Create a small placeholder in the grid
        let display_size = 120.0;
        let aspect_ratio = size.x as f32 / size.y as f32;
        let display_height = display_size;
        let display_width = display_height * aspect_ratio;

        // Try to display actual render target in grid
        match texture_id {
            Some(texture_id) => {
                // Successfully got egui texture ID, display the actual image
                ui.image(egui::load::SizedTexture::new(
                    texture_id,
                    egui::Vec2::new(display_width, display_height),
                ));
            }
            None => {
                // Fallback to visualization if texture not registered with egui
                match images.get(target) {
                    Some(image) => {
                        if let Some(data) = &image.data {
                            let data_size = data.len();
                            let _pixel_count = data_size / 4;

                            // Use different colors for different render targets
                            let color = match label {
                                "Floor" => egui::Color32::from_rgb(100, 150, 100),
                                "Walls" => egui::Color32::from_rgb(150, 100, 100),
                                "Objects" => egui::Color32::from_rgb(100, 100, 150),
                                _ => egui::Color32::from_rgb(120, 120, 120),
                            };

                            let (rect, _) = ui.allocate_exact_size(
                                egui::Vec2::new(display_width, display_height),
                                egui::Sense::hover(),
                            );

                            ui.painter()
                                .rect_filled(rect, egui::CornerRadius::same(4), color);

                            // Add text overlay
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{}×{}", size.x, size.y),
                                egui::FontId::default(),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                    _ => {
                        ui.label("Not avail");
                    }
                }
            }
        }
    } else {
        ui.label("Not available");
    }
}
