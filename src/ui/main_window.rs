//! The systems that update the main window.

use bevy::ecs::entity::unique_slice::Windows;
use super::config::{MeshColor, WfColor};
use super::right_panel::ElementTypesRes;
use super::{camera::ProjectionType, top_panel::SectionState};
use crate::mesh::Renderable;
use crate::Concrete;

use bevy::prelude::*;
use bevy_egui::EguiSettings;
use miratope_core::abs::Ranked;

/// The plugin in charge of the Miratope main window, and of drawing the
/// polytope onto it.
pub struct MainWindowPlugin;

impl Plugin for MainWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, update_visible.system())
            .add_system(update_scale_factor.system())
            .add_system_to_stage(CoreStage::PostUpdate, update_changed_polytopes.system())
            .add_system_to_stage(CoreStage::PostUpdate, update_changed_color.system())
            .init_resource::<PolyName>();
    }
}

#[derive(Resource)]
pub struct PolyName(pub String);

impl Default for PolyName {
    fn default() -> PolyName {
        PolyName("default".to_string())
    }
}

pub fn update_visible(
    keyboard: Res<'_, Input<KeyCode>>,
    mut polies_vis: Query<'_, '_, &mut Visible, With<Concrete>>,
    mut wfs_vis: Query<'_, '_, &mut Visible, Without<Concrete>>,
) {
    if keyboard.get_pressed().count() == 1 {
        if keyboard.just_pressed(KeyCode::KeyV) {
            if let Some(mut visible) = polies_vis.iter_mut().next() {
                let vis = visible.is_visible;
                visible.is_visible = !vis;
            }
        }

        if keyboard.just_pressed(KeyCode::KeyB) {
            if let Some(mut visible) = wfs_vis.iter_mut().next() {
                let vis = visible.is_visible;
                visible.is_visible = !vis;
            }
        }
    }
}

/// Resizes the UI when the screen is resized.
pub fn update_scale_factor(mut egui_settings: ResMut<'_, EguiSettings>, windows: Res<'_, Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.0 / window.scale_factor();
    }
}

/// Updates polytopes after an operation.
pub fn update_changed_polytopes(
    mut meshes: ResMut<'_, Assets<Mesh>>,
    polies: Query<'_, '_, (&Concrete, &Handle<Mesh>, &Children), Changed<Concrete>>,
    wfs: Query<'_, '_, &Handle<Mesh>, Without<Concrete>>,
    mut windows: ResMut<'_, Windows>,
    mut section_state: ResMut<'_, SectionState>,
    mut element_types: ResMut<'_, ElementTypesRes>,
    name: Res<'_, PolyName>,

    orthogonal: Res<'_, ProjectionType>,
) {
    for (poly, mesh_handle, children) in polies.iter() {
        if cfg!(debug_assertions) {
            poly.assert_valid();
        }

        if !element_types.main_updating {
            element_types.main = false;
        } else {
            element_types.main_updating = false;
        }

        *meshes.get_mut(mesh_handle).unwrap() = poly.mesh(*orthogonal);

        // Updates all wireframes.
        for child in children.iter() {
            if let Ok(wf_handle) = wfs.get_component::<Handle<Mesh>>(*child) {
                *meshes.get_mut(wf_handle).unwrap() = poly.wireframe(*orthogonal);
            }
        }

        // We reset the cross-section view if we didn't use it to change the polytope.
        if !section_state.is_changed() {
            section_state.close();
        }

        windows
            .get_primary_mut()
            .unwrap()
            .set_title(format!("{} - Miratope v{}", name.0, env!("CARGO_PKG_VERSION")));

    }
}

pub fn update_changed_color(
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
    mut polies: Query<'_, '_, &Handle<StandardMaterial>, With<Concrete>>,
    mut wfs: Query<'_, '_, &Handle<StandardMaterial>, Without<Concrete>>,
    mesh_color: Res<'_, MeshColor>,
    wf_color: Res<'_, WfColor>,
) {
    if let Some(material_handle) = polies.iter_mut().next() {
        *materials.get_mut(material_handle).unwrap() = StandardMaterial {
            base_color: mesh_color.0,
            metallic: 0.0,
            ..Default::default()
        };
    }
    if let Some(wf_handle) = wfs.iter_mut().next() {
        *materials.get_mut(wf_handle).unwrap() = wf_color.0.into()
    }
}