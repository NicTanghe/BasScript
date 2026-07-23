use std::{collections::HashMap, time::Instant};

use bevy::{
    input::{
        ButtonState,
        pen::{PenAction, PenButton, PenData, PenId, PenInput, PenPressure, PenToolKind},
    },
    prelude::*,
    ui::UiGlobalTransform,
    window::PrimaryWindow,
};
use vector_stroke_render::{
    CanvasExtent, CanvasId, DocumentLimits, LayerId, PenTilt, StrokeDocument, StrokeId,
    StrokePoint, StrokeResampler, VectorCanvasView, VectorStrokeInputBlocker, VectorStrokeSettings,
    VectorStrokeTarget, load_ron_file, save_ron_atomic,
};

use super::*;

const DRAWING_FRONT_MATTER_KEY: &str = "drawings";
const DRAWING_SIDECAR_SUFFIX: &str = ".ink.ron";
pub(crate) const DRAWING_RENDER_LAYER: usize = 7;

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct DrawingOverlayRoot {
    pub(crate) kind: PanelKind,
}

#[derive(Component)]
pub(crate) struct DrawingRenderCamera;

#[derive(Component)]
pub(crate) struct DrawingVectorView;

#[derive(Resource, Debug)]
pub(crate) struct DrawingSession {
    source_path: Option<PathBuf>,
    sidecar_path: Option<PathBuf>,
    canvas: Option<CanvasId>,
    layer: Option<LayerId>,
    view_entity: Option<Entity>,
    allowed_viewport: Option<Rect>,
    surface_origin_viewport: Option<Vec2>,
    surface_zoom: f32,
    surface_extent: CanvasExtent,
    last_saved_revision: Option<u64>,
    save_requested: bool,
    save_blocked: bool,
}

impl Default for DrawingSession {
    fn default() -> Self {
        Self {
            source_path: None,
            sidecar_path: None,
            canvas: None,
            layer: None,
            view_entity: None,
            allowed_viewport: None,
            surface_origin_viewport: None,
            surface_zoom: 1.0,
            surface_extent: CanvasExtent::new(0.0, 0.0),
            last_saved_revision: None,
            save_requested: false,
            save_blocked: false,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct DrawingPenState {
    contacts: HashMap<PenId, DrawingPenContact>,
}

enum DrawingPenContact {
    Draw {
        stroke: StrokeId,
        resampler: StrokeResampler,
        started: Instant,
    },
    Erase {
        canvas: CanvasId,
    },
}

pub(crate) fn drawing_file_is_supported(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    matches!(
        extension.as_deref(),
        Some("fountain") | Some("md") | Some("markdown")
    )
}

pub(crate) fn handle_drawing_mode_toggle(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<DrawingModeToggle>)>,
    mut state: ResMut<EditorState>,
    mut session: ResMut<DrawingSession>,
) {
    for interaction in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if state.drawing_mode_enabled {
            state.drawing_mode_enabled = false;
            state.status_message = "Drawing hidden.".to_string();
            continue;
        }

        if !drawing_file_is_supported(&state.paths.load_path) {
            state.status_message =
                "Drawing is available only for Fountain and Markdown files.".to_string();
            continue;
        }
        if session.save_blocked {
            state.status_message =
                "Drawing is unavailable because the linked RON sidecar could not be loaded."
                    .to_string();
            continue;
        }

        let link = document_drawing_link(&state.document, state.document_format)
            .filter(|link| !link.trim().is_empty())
            .unwrap_or_else(|| drawing_link_for_source(&state.paths.load_path));
        let snapshot = state.history_snapshot();
        let document_format = state.document_format;
        if ensure_document_drawing_link(&mut state.document, document_format, &link) {
            state.push_undo_snapshot(snapshot);
            state.redo_history.clear();
            state.reparse();
            state.cursor.position = state.document.clamp_position(state.cursor.position);
            state.cursor.preferred_column = state
                .cursor
                .preferred_column
                .min(state.document.line_len_chars(state.cursor.position.line));
        }

        state.drawing_mode_enabled = true;
        state.focused_panel = PanelKind::Processed;
        state.selection_anchor = None;
        session.save_requested = true;
        state.status_message = format!(
            "Drawing visible. Pen contact draws; the pen eraser erases. Linked {}.",
            link
        );
    }
}

pub(crate) fn handle_drawing_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    mut document: ResMut<StrokeDocument>,
) {
    if !state.drawing_mode_enabled {
        return;
    }

    if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::Redo)) {
        state.status_message = match document.redo() {
            Ok(()) => "Drawing redo.".to_string(),
            Err(_) => "Nothing to redo in the drawing.".to_string(),
        };
        return;
    }

    if shortcut_just_pressed(&keys, state.keybinds.binding(ShortcutAction::Undo)) {
        state.status_message = match document.undo() {
            Ok(()) => "Drawing undo.".to_string(),
            Err(_) => "Nothing to undo in the drawing.".to_string(),
        };
    }
}

pub(crate) fn sync_drawing_session(
    mut commands: Commands,
    mut state: ResMut<EditorState>,
    mut document: ResMut<StrokeDocument>,
    mut target: ResMut<VectorStrokeTarget>,
    mut session: ResMut<DrawingSession>,
) {
    let source_path = state.paths.load_path.clone();
    if session.source_path.as_ref() == Some(&source_path) {
        return;
    }

    let carry_current_drawing = state.drawing_mode_enabled
        && session.source_path.is_some()
        && !document.has_active_strokes();
    let carried_document = carry_current_drawing.then(|| document.clone());
    state.drawing_mode_enabled = false;
    target.clear();
    if let Some(view_entity) = session.view_entity.take() {
        commands.entity(view_entity).despawn();
    }
    session.source_path = Some(source_path.clone());
    session.sidecar_path = None;
    session.canvas = None;
    session.layer = None;
    session.allowed_viewport = None;
    session.surface_origin_viewport = None;
    session.surface_extent = CanvasExtent::new(0.0, 0.0);
    session.last_saved_revision = None;
    session.save_requested = false;
    session.save_blocked = false;

    if !drawing_file_is_supported(&source_path) {
        *document = StrokeDocument::default();
        return;
    }

    let link = document_drawing_link(&state.document, state.document_format)
        .filter(|link| !link.trim().is_empty())
        .unwrap_or_else(|| drawing_link_for_source(&source_path));
    let sidecar_path = resolve_drawing_link(&source_path, &link);
    let sidecar_exists = sidecar_path.is_file();
    let mut next_document = if sidecar_exists {
        match load_ron_file(&sidecar_path, DocumentLimits::default()) {
            Ok(document) => document,
            Err(error) => {
                session.save_blocked = true;
                state.status_message = format!(
                    "Could not load drawing {}: {error}. The file was left untouched.",
                    sidecar_path.display()
                );
                *document = StrokeDocument::default();
                return;
            }
        }
    } else {
        carried_document.unwrap_or_else(|| StrokeDocument::new(source_path.to_string_lossy()))
    };

    let (canvas, layer) = ensure_drawing_surface(&mut next_document);
    let loaded_revision = sidecar_exists.then_some(next_document.revision());
    *document = next_document;

    session.sidecar_path = Some(sidecar_path);
    session.canvas = Some(canvas);
    session.layer = Some(layer);
    session.view_entity = Some(
        commands
            .spawn((
                VectorCanvasView::new(canvas),
                DrawingVectorView,
                RenderLayers::layer(DRAWING_RENDER_LAYER),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id(),
    );
    session.last_saved_revision = loaded_revision;
    session.save_requested = carry_current_drawing && !sidecar_exists;
    state.drawing_mode_enabled = carry_current_drawing;
}

pub(crate) fn sync_drawing_surface(
    mut state: ResMut<EditorState>,
    mut document: ResMut<StrokeDocument>,
    mut session: ResMut<DrawingSession>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    canvas_query: Query<(&PanelCanvas, &ComputedNode, &UiGlobalTransform)>,
    paper_query: Query<(&PanelPaper, &Node, &Visibility), Without<DrawingOverlayRoot>>,
    mut root_query: Query<
        (&DrawingOverlayRoot, &mut Node),
        (Without<PanelPaper>, Without<PanelCanvas>),
    >,
    mut view_query: Query<
        (&VectorCanvasView, &mut Transform, &mut Visibility),
        (With<DrawingVectorView>, Without<PanelPaper>),
    >,
    mut camera_query: Query<&mut Camera, With<DrawingRenderCamera>>,
) {
    for (root, mut node) in root_query.iter_mut() {
        if root.kind == PanelKind::Processed {
            node.display = Display::None;
        }
    }
    for (_, _, mut visibility) in view_query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    for mut camera in camera_query.iter_mut() {
        camera.is_active = false;
        camera.viewport = None;
    }

    let active = state.drawing_mode_enabled
        && drawing_file_is_supported(&state.paths.load_path)
        && state.panel_visible(PanelKind::Processed)
        && !session.save_blocked;
    session.allowed_viewport = None;
    session.surface_origin_viewport = None;
    if !active {
        return;
    }

    let Some(canvas_id) = session.canvas else {
        return;
    };
    let Some((_, mut vector_transform, mut vector_visibility)) = view_query
        .iter_mut()
        .find(|(view, _, _)| view.canvas == canvas_id)
    else {
        return;
    };
    let Ok(window) = window_query.single() else {
        return;
    };
    let Some((_, panel_computed, panel_transform)) = canvas_query
        .iter()
        .find(|(panel, _, _)| panel.kind == PanelKind::Processed)
    else {
        return;
    };
    let Some((_, paper_node, _)) = paper_query.iter().find(|(paper, _, visibility)| {
        paper.kind == PanelKind::Processed && paper.slot == 0 && **visibility == Visibility::Visible
    }) else {
        return;
    };

    let panel_size = panel_computed.size() * panel_computed.inverse_scale_factor();
    let layout = processed_page_layout(panel_size, &state);
    let page_step_lines = layout.page_step_lines.max(1);
    let capacity = page_step_lines
        .saturating_mul(PROCESSED_PAPER_CAPACITY)
        .max(1);
    let all_lines = processed_display_lines(
        &mut state,
        layout.wrap_columns,
        layout.lines_per_page,
        layout.spacer_lines,
    );
    let view = build_processed_view(
        &all_lines,
        state.processed_top_visual,
        page_step_lines,
        capacity,
    );
    let first_visible_page = view.start_index / page_step_lines;
    let page_count = processed_page_count_for_lines(&all_lines, page_step_lines);
    let base_page_step = page_step_lines as f32 * LINE_HEIGHT;
    let requested_height = if state.processed_paginated {
        (page_count as f32 * base_page_step - PAGE_GAP).max(A4_HEIGHT_POINTS)
    } else {
        let content_height = all_lines
            .iter()
            .map(|line| processed_visual_line_height_units(&state, line))
            .sum::<f32>()
            * LINE_HEIGHT;
        (state.page_margin_top + content_height + state.page_margin_bottom).max(A4_HEIGHT_POINTS)
    };
    let required = drawing_required_extent(&document, canvas_id);
    let extent = CanvasExtent::new(
        A4_WIDTH_POINTS.max(required.width),
        requested_height.max(required.height),
    );
    if document.canvas(canvas_id).is_ok_and(|canvas| {
        (canvas.extent.width - extent.width).abs() > 0.01
            || (canvas.extent.height - extent.height).abs() > 0.01
    }) {
        let _ = document.set_canvas_extent(canvas_id, extent);
    }

    let (Val::Px(paper_left), Val::Px(first_paper_top)) = (paper_node.left, paper_node.top) else {
        return;
    };
    let preceding_height = if state.processed_paginated {
        first_visible_page as f32 * base_page_step * state.zoom
    } else {
        all_lines
            .get(..first_visible_page.saturating_mul(page_step_lines))
            .unwrap_or(&[])
            .iter()
            .map(|line| processed_visual_line_height_units(&state, line))
            .sum::<f32>()
            * LINE_HEIGHT
            * state.zoom
    };
    let origin_top = first_paper_top - preceding_height;

    let (_, _, panel_translation_physical) = panel_transform.to_scale_angle_translation();
    let panel_top_left =
        panel_translation_physical * panel_computed.inverse_scale_factor() - panel_size * 0.5;
    let viewport_top_left = panel_top_left + Vec2::new(paper_left, origin_top);
    session.surface_origin_viewport = Some(viewport_top_left);
    session.surface_zoom = state.zoom;
    session.surface_extent = extent;

    let panel_rect = Rect::from_corners(panel_top_left, panel_top_left + panel_size);
    session.allowed_viewport = (panel_rect.max.x > panel_rect.min.x
        && panel_rect.max.y > panel_rect.min.y)
        .then_some(panel_rect);

    let panel_size_physical = panel_computed.size();
    let panel_top_left_physical = panel_translation_physical - panel_size_physical * 0.5;
    let target_size = UVec2::new(
        window.resolution.physical_width(),
        window.resolution.physical_height(),
    );
    let Some(viewport) =
        drawing_camera_viewport(panel_top_left_physical, panel_size_physical, target_size)
    else {
        return;
    };
    let Ok(mut camera) = camera_query.single_mut() else {
        return;
    };

    let os_scale = window.scale_factor().max(f32::EPSILON);
    let target_scale = panel_computed.inverse_scale_factor().recip();
    let paper_origin_physical =
        panel_top_left_physical + Vec2::new(paper_left, origin_top) * target_scale;
    let Some(transform) = drawing_vector_view_transform(
        paper_origin_physical,
        &viewport,
        os_scale,
        state.zoom,
        target_scale,
    ) else {
        return;
    };

    *vector_transform = transform;
    *vector_visibility = Visibility::Visible;
    camera.viewport = Some(viewport);
    camera.is_active = true;
}

pub(crate) fn hide_drawing_render_overlay(
    mut view_query: Query<&mut Visibility, With<DrawingVectorView>>,
    mut camera_query: Query<&mut Camera, With<DrawingRenderCamera>>,
) {
    for mut visibility in view_query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    for mut camera in camera_query.iter_mut() {
        camera.is_active = false;
        camera.viewport = None;
    }
}

pub(crate) fn sync_drawing_input(
    window_query: Query<&Window, With<PrimaryWindow>>,
    toggle_query: Query<(&ComputedNode, &UiGlobalTransform), With<ProcessedOverlayToggleGroup>>,
    ui_scale: Res<UiScale>,
    state: Res<EditorState>,
    session: Res<DrawingSession>,
    mut target: ResMut<VectorStrokeTarget>,
    mut blocker: ResMut<VectorStrokeInputBlocker>,
    mut stroke_settings: ResMut<VectorStrokeSettings>,
) {
    // BasScript maps tablet coordinates directly from the rendered UI paper.
    // Disable the renderer's world-camera input adapter (and mouse drawing)
    // so one physical pen event can create only one authoritative stroke.
    stroke_settings.enable_pen = false;
    stroke_settings.enable_mouse = false;
    target.clear();

    let Ok(window) = window_query.single() else {
        blocker.clear();
        return;
    };
    let ui_scale = ui_scale.0.max(f32::EPSILON);
    let viewport_size = window.size() / ui_scale;
    let window_rect = Rect::from_corners(Vec2::ZERO, viewport_size);
    let active = state.drawing_mode_enabled
        && !session.save_blocked
        && session.allowed_viewport.is_some()
        && session.surface_origin_viewport.is_some()
        && session.surface_zoom.is_finite()
        && session.surface_zoom > f32::EPSILON
        && session.surface_extent.width > 0.0
        && session.surface_extent.height > 0.0
        && session.canvas.is_some()
        && session.layer.is_some();
    if !active {
        blocker.set_regions([window_rect]);
        return;
    }

    let allowed = session
        .allowed_viewport
        .expect("active drawing viewport")
        .intersect(window_rect);
    let mut regions = blocking_regions_around(allowed, viewport_size);
    regions.extend(toggle_query.iter().filter_map(|(computed, transform)| {
        let inverse_scale = computed.inverse_scale_factor();
        let size = computed.size() * inverse_scale;
        let (_, _, translation) = transform.to_scale_angle_translation();
        let top_left = translation * inverse_scale - size * 0.5;
        (size.x > 0.0 && size.y > 0.0).then(|| Rect::from_corners(top_left, top_left + size))
    }));
    blocker.set_regions(regions);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_drawing_pen_input(
    mut messages: MessageReader<PenInput>,
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    state: Res<EditorState>,
    session: Res<DrawingSession>,
    blocker: Res<VectorStrokeInputBlocker>,
    settings: Res<VectorStrokeSettings>,
    mut pen_state: ResMut<DrawingPenState>,
    mut document: ResMut<StrokeDocument>,
) {
    let active = state.drawing_mode_enabled
        && !session.save_blocked
        && session.canvas.is_some()
        && session.layer.is_some()
        && session.surface_origin_viewport.is_some()
        && session.surface_zoom > f32::EPSILON
        && ui_scale.0.is_finite()
        && ui_scale.0 > f32::EPSILON;
    if !active {
        for (_, contact) in pen_state.contacts.drain() {
            finish_drawing_pen_contact(contact, &mut document);
        }
        for _ in messages.read() {}
        return;
    }

    let Some(primary_window) = primary_window_query.iter().next() else {
        return;
    };
    let canvas = session.canvas.expect("active drawing canvas");
    let layer = session.layer.expect("active drawing layer");

    for message in messages.read() {
        if message.pen.window != primary_window {
            continue;
        }

        match &message.action {
            PenAction::Button {
                button: PenButton::Contact,
                state: ButtonState::Pressed,
                data,
            } => {
                if let Some(contact) = pen_state.contacts.remove(&message.pen.device) {
                    finish_drawing_pen_contact(contact, &mut document);
                }
                let started = Instant::now();
                let Some(point) =
                    drawing_point_from_pen(message, data, ui_scale.0, &session, &blocker, started)
                else {
                    continue;
                };

                if message.pen.tool == PenToolKind::Eraser {
                    let _ =
                        document.erase_strokes(canvas, point.position(), settings.eraser_radius);
                    pen_state
                        .contacts
                        .insert(message.pen.device, DrawingPenContact::Erase { canvas });
                } else if let Ok(stroke) =
                    document.begin_stroke(canvas, layer, settings.pen_style.clone(), point)
                {
                    pen_state.contacts.insert(
                        message.pen.device,
                        DrawingPenContact::Draw {
                            stroke,
                            resampler: StrokeResampler::new(
                                point,
                                settings.pen_style.clone(),
                                settings.resampling,
                            ),
                            started,
                        },
                    );
                }
            }
            PenAction::Moved(data) => {
                let Some(mut contact) = pen_state.contacts.remove(&message.pen.device) else {
                    continue;
                };
                let started = contact.started().unwrap_or_else(Instant::now);
                let Some(point) =
                    drawing_point_from_pen(message, data, ui_scale.0, &session, &blocker, started)
                else {
                    finish_drawing_pen_contact(contact, &mut document);
                    continue;
                };
                update_drawing_pen_contact(
                    &mut contact,
                    point,
                    settings.eraser_radius,
                    &mut document,
                );
                pen_state.contacts.insert(message.pen.device, contact);
            }
            PenAction::Button {
                button: PenButton::Contact,
                state: ButtonState::Released,
                data,
            } => {
                let Some(mut contact) = pen_state.contacts.remove(&message.pen.device) else {
                    continue;
                };
                let started = contact.started().unwrap_or_else(Instant::now);
                if let Some(point) =
                    drawing_point_from_pen(message, data, ui_scale.0, &session, &blocker, started)
                {
                    update_drawing_pen_contact(
                        &mut contact,
                        point,
                        settings.eraser_radius,
                        &mut document,
                    );
                }
                finish_drawing_pen_contact(contact, &mut document);
            }
            PenAction::Left => {
                if let Some(contact) = pen_state.contacts.remove(&message.pen.device) {
                    finish_drawing_pen_contact(contact, &mut document);
                }
            }
            PenAction::Entered
            | PenAction::Button {
                button: PenButton::Barrel | PenButton::Other(_),
                ..
            } => {}
        }
    }
}

pub(crate) fn save_drawing_sidecar(
    mut state: ResMut<EditorState>,
    document: Res<StrokeDocument>,
    mut session: ResMut<DrawingSession>,
) {
    let drawing_was_initialized = state.drawing_mode_enabled
        || session.save_requested
        || session.last_saved_revision.is_some();
    if !drawing_was_initialized
        || session.save_blocked
        || document.has_active_strokes()
        || (!session.save_requested && session.last_saved_revision == Some(document.revision()))
    {
        return;
    }
    let Some(path) = session.sidecar_path.clone() else {
        return;
    };

    match save_ron_atomic(&path, &document) {
        Ok(()) => {
            session.last_saved_revision = Some(document.revision());
            session.save_requested = false;
        }
        Err(error) => {
            state.status_message = format!("Could not save drawing {}: {error}", path.display());
        }
    }
}

impl DrawingPenContact {
    fn started(&self) -> Option<Instant> {
        match self {
            Self::Draw { started, .. } => Some(*started),
            Self::Erase { .. } => None,
        }
    }
}

fn update_drawing_pen_contact(
    contact: &mut DrawingPenContact,
    point: StrokePoint,
    eraser_radius: f32,
    document: &mut StrokeDocument,
) {
    match contact {
        DrawingPenContact::Draw {
            stroke, resampler, ..
        } => {
            let stroke = *stroke;
            resampler.push(point, |point| {
                let _ = document.append_point(stroke, point);
            });
        }
        DrawingPenContact::Erase { canvas } => {
            let _ = document.erase_strokes(*canvas, point.position(), eraser_radius);
        }
    }
}

fn finish_drawing_pen_contact(contact: DrawingPenContact, document: &mut StrokeDocument) {
    if let DrawingPenContact::Draw {
        stroke,
        mut resampler,
        ..
    } = contact
    {
        resampler.finish(|point| {
            let _ = document.append_point(stroke, point);
        });
        let _ = document.end_stroke(stroke);
    }
}

fn drawing_point_from_pen(
    message: &PenInput,
    data: &PenData,
    ui_scale: f32,
    session: &DrawingSession,
    blocker: &VectorStrokeInputBlocker,
    started: Instant,
) -> Option<StrokePoint> {
    let viewport = drawing_viewport_to_ui(message.pen.position?, ui_scale)?;
    if blocker.blocks(viewport) {
        return None;
    }
    let position = drawing_viewport_to_document(
        viewport,
        session.surface_origin_viewport?,
        session.surface_zoom,
    )?;
    let pressure = match data.pressure {
        Some(PenPressure::Normalized(value)) => value as f32,
        Some(PenPressure::Calibrated {
            force,
            max_possible_force,
        }) if max_possible_force > 0.0 => (force / max_possible_force) as f32,
        Some(PenPressure::Calibrated { .. }) | None => 1.0,
    }
    .clamp(0.0, 1.0);
    Some(StrokePoint {
        x: position.x,
        y: position.y,
        pressure,
        tilt: data.tilt.map(|tilt| PenTilt {
            x: tilt.x as f32,
            y: tilt.y as f32,
        }),
        twist: data.twist.map(|twist| twist as f32),
        elapsed_ms: Some(elapsed_millis(started.elapsed())),
    })
}

fn drawing_viewport_to_ui(viewport: Vec2, ui_scale: f32) -> Option<Vec2> {
    if !viewport.is_finite() || !ui_scale.is_finite() || ui_scale <= f32::EPSILON {
        return None;
    }
    Some(viewport / ui_scale)
}

fn drawing_viewport_to_document(viewport: Vec2, surface_origin: Vec2, zoom: f32) -> Option<Vec2> {
    if !zoom.is_finite() || zoom <= f32::EPSILON {
        return None;
    }
    let local = (viewport - surface_origin) / zoom;
    local.is_finite().then_some(local)
}

fn drawing_camera_viewport(
    panel_top_left: Vec2,
    panel_size: Vec2,
    target_size: UVec2,
) -> Option<Viewport> {
    if !panel_top_left.is_finite()
        || !panel_size.is_finite()
        || panel_size.x <= 0.0
        || panel_size.y <= 0.0
        || target_size.x == 0
        || target_size.y == 0
    {
        return None;
    }

    let target_size_f32 = target_size.as_vec2();
    let min = panel_top_left.max(Vec2::ZERO).floor();
    let max = (panel_top_left + panel_size).min(target_size_f32).ceil();
    if max.x <= min.x || max.y <= min.y {
        return None;
    }

    let physical_position = min.as_uvec2();
    let physical_end = max.as_uvec2().min(target_size);
    let physical_size = physical_end.saturating_sub(physical_position);
    (physical_size.x > 0 && physical_size.y > 0).then_some(Viewport {
        physical_position,
        physical_size,
        ..default()
    })
}

fn drawing_vector_view_transform(
    paper_origin_physical: Vec2,
    viewport: &Viewport,
    os_scale: f32,
    document_zoom: f32,
    target_scale: f32,
) -> Option<Transform> {
    if !paper_origin_physical.is_finite()
        || !os_scale.is_finite()
        || os_scale <= f32::EPSILON
        || !document_zoom.is_finite()
        || document_zoom <= f32::EPSILON
        || !target_scale.is_finite()
        || target_scale <= f32::EPSILON
    {
        return None;
    }

    let paper_origin_in_viewport =
        (paper_origin_physical - viewport.physical_position.as_vec2()) / os_scale;
    let viewport_logical_size = viewport.physical_size.as_vec2() / os_scale;
    let world_origin = Vec2::new(
        paper_origin_in_viewport.x - viewport_logical_size.x * 0.5,
        viewport_logical_size.y * 0.5 - paper_origin_in_viewport.y,
    );
    let render_scale = document_zoom * target_scale / os_scale;
    Some(
        Transform::from_xyz(world_origin.x, world_origin.y, 0.0).with_scale(Vec3::new(
            render_scale,
            render_scale,
            1.0,
        )),
    )
}

fn elapsed_millis(duration: std::time::Duration) -> u32 {
    duration.as_millis().min(u32::MAX as u128) as u32
}

fn ensure_drawing_surface(document: &mut StrokeDocument) -> (CanvasId, LayerId) {
    if let Some(canvas) = document.canvases().first()
        && let Some(layer) = canvas.layers.first()
    {
        return (canvas.id, layer.id);
    }

    let canvas = document
        .create_canvas(CanvasExtent::new(A4_WIDTH_POINTS, A4_HEIGHT_POINTS), None)
        .expect("default drawing extent is valid");
    let layer = document.canvas(canvas).expect("new drawing canvas").layers[0].id;
    (canvas, layer)
}

fn drawing_required_extent(document: &StrokeDocument, canvas: CanvasId) -> CanvasExtent {
    let mut width = A4_WIDTH_POINTS;
    let mut height = A4_HEIGHT_POINTS;
    if let Ok(canvas) = document.canvas(canvas) {
        for stroke in canvas.layers.iter().flat_map(|layer| layer.strokes.iter()) {
            for point in &stroke.points {
                let radius = stroke.style.half_width_at(point.pressure);
                width = width.max(point.x + radius);
                height = height.max(point.y + radius);
            }
        }
    }
    CanvasExtent::new(width, height)
}

fn blocking_regions_around(allowed: Rect, window_size: Vec2) -> Vec<Rect> {
    let mut regions = Vec::with_capacity(4);
    let candidates = [
        Rect::from_corners(Vec2::ZERO, Vec2::new(window_size.x, allowed.min.y)),
        Rect::from_corners(
            Vec2::new(0.0, allowed.max.y),
            Vec2::new(window_size.x, window_size.y),
        ),
        Rect::from_corners(
            Vec2::new(0.0, allowed.min.y),
            Vec2::new(allowed.min.x, allowed.max.y),
        ),
        Rect::from_corners(
            Vec2::new(allowed.max.x, allowed.min.y),
            Vec2::new(window_size.x, allowed.max.y),
        ),
    ];
    regions.extend(
        candidates
            .into_iter()
            .filter(|region| region.max.x > region.min.x && region.max.y > region.min.y),
    );
    regions
}

fn drawing_link_for_source(source_path: &Path) -> String {
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document");
    format!("{file_name}{DRAWING_SIDECAR_SUFFIX}")
}

fn resolve_drawing_link(source_path: &Path, link: &str) -> PathBuf {
    let linked = PathBuf::from(link);
    if linked.is_absolute() {
        linked
    } else {
        source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(linked)
    }
}

fn document_drawing_link(document: &Document, format: DocumentFormat) -> Option<String> {
    match format {
        DocumentFormat::Markdown => markdown_drawing_link(document),
        DocumentFormat::Fountain => fountain_drawing_link(document),
        DocumentFormat::Canvas => None,
    }
}

fn markdown_drawing_link(document: &Document) -> Option<String> {
    let lines = document.lines();
    let first = lines.first()?;
    if !line_is_front_matter_delimiter(first, true) {
        return None;
    }
    for line in lines.iter().skip(1) {
        if line_is_front_matter_delimiter(line, false) {
            break;
        }
        if let Some((key, value)) = line.trim().split_once(':')
            && key.trim().eq_ignore_ascii_case(DRAWING_FRONT_MATTER_KEY)
        {
            return Some(markdown_yaml_scalar(value));
        }
    }
    None
}

fn fountain_drawing_link(document: &Document) -> Option<String> {
    for line in document
        .lines()
        .iter()
        .take_while(|line| !line.trim().is_empty())
    {
        if let Some((key, value)) = line.trim().split_once(':')
            && key.trim().eq_ignore_ascii_case("Drawings")
        {
            return Some(markdown_yaml_scalar(value));
        }
    }
    None
}

fn ensure_document_drawing_link(
    document: &mut Document,
    format: DocumentFormat,
    link: &str,
) -> bool {
    match format {
        DocumentFormat::Markdown => ensure_markdown_drawing_link(document, link),
        DocumentFormat::Fountain => ensure_fountain_drawing_link(document, link),
        DocumentFormat::Canvas => false,
    }
}

fn ensure_markdown_drawing_link(document: &mut Document, link: &str) -> bool {
    let mut lines = document.lines().to_vec();
    let has_front_matter = lines
        .first()
        .is_some_and(|line| line_is_front_matter_delimiter(line, true));
    if has_front_matter {
        let closing =
            lines.iter().enumerate().skip(1).find_map(|(index, line)| {
                line_is_front_matter_delimiter(line, false).then_some(index)
            });
        if let Some(closing) = closing {
            for line in lines.iter_mut().take(closing).skip(1) {
                let Some((key, _)) = line.trim().split_once(':') else {
                    continue;
                };
                if key.trim().eq_ignore_ascii_case(DRAWING_FRONT_MATTER_KEY) {
                    let replacement = format!("{DRAWING_FRONT_MATTER_KEY}: {}", yaml_scalar(link));
                    if *line == replacement {
                        return false;
                    }
                    *line = replacement;
                    *document = Document::from_text(&lines.join("\n"));
                    return true;
                }
            }
            lines.insert(
                closing,
                format!("{DRAWING_FRONT_MATTER_KEY}: {}", yaml_scalar(link)),
            );
            *document = Document::from_text(&lines.join("\n"));
            return true;
        }
    }

    let mut prefixed = vec![
        "---".to_string(),
        format!("{DRAWING_FRONT_MATTER_KEY}: {}", yaml_scalar(link)),
        "---".to_string(),
    ];
    prefixed.extend(lines);
    *document = Document::from_text(&prefixed.join("\n"));
    true
}

fn ensure_fountain_drawing_link(document: &mut Document, link: &str) -> bool {
    let mut lines = document.lines().to_vec();
    let header_end = lines
        .iter()
        .position(|line| line.trim().is_empty())
        .unwrap_or(lines.len());
    for line in lines.iter_mut().take(header_end) {
        let Some((key, _)) = line.trim().split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("Drawings") {
            let replacement = format!("Drawings: {link}");
            if *line == replacement {
                return false;
            }
            *line = replacement;
            *document = Document::from_text(&lines.join("\n"));
            return true;
        }
    }

    let title_page_present = lines
        .first()
        .is_some_and(|line| line.split_once(':').is_some() && !line_is_fountain_scene(line));
    if title_page_present {
        lines.insert(header_end, format!("Drawings: {link}"));
    } else {
        lines.insert(0, String::new());
        lines.insert(0, format!("Drawings: {link}"));
    }
    *document = Document::from_text(&lines.join("\n"));
    true
}

fn line_is_fountain_scene(line: &str) -> bool {
    let upper = line.trim_start().to_ascii_uppercase();
    ["INT.", "EXT.", "EST.", "INT/EXT.", "I/E."]
        .iter()
        .any(|prefix| upper.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::pen::{PenInfo, PenTilt as InputPenTilt};

    #[test]
    fn recognizes_only_fountain_and_markdown_paths() {
        assert!(drawing_file_is_supported(Path::new("script.fountain")));
        assert!(drawing_file_is_supported(Path::new("notes.MD")));
        assert!(drawing_file_is_supported(Path::new("notes.markdown")));
        assert!(!drawing_file_is_supported(Path::new("board.canvas")));
        assert!(!drawing_file_is_supported(Path::new("notes.txt")));
    }

    #[test]
    fn markdown_link_is_added_and_read_without_rewriting_existing_metadata() {
        let mut document =
            Document::from_text("---\ntarget: eoghan\ncustom: value\n---\n# Notes\n");
        assert!(ensure_markdown_drawing_link(
            &mut document,
            "notes.md.ink.ron"
        ));
        assert_eq!(
            document.to_text(),
            "---\ntarget: eoghan\ncustom: value\ndrawings: notes.md.ink.ron\n---\n# Notes\n"
        );
        assert_eq!(
            markdown_drawing_link(&document).as_deref(),
            Some("notes.md.ink.ron")
        );
        assert!(!ensure_markdown_drawing_link(
            &mut document,
            "notes.md.ink.ron"
        ));
    }

    #[test]
    fn markdown_without_front_matter_gets_a_drawing_link() {
        let mut document = Document::from_text("# Notes\nBody\n");
        assert!(ensure_markdown_drawing_link(
            &mut document,
            "notes.md.ink.ron"
        ));
        assert_eq!(
            document.to_text(),
            "---\ndrawings: notes.md.ink.ron\n---\n# Notes\nBody\n"
        );
    }

    #[test]
    fn fountain_uses_a_title_page_drawing_field() {
        let mut document =
            Document::from_text("Title: Small Things\nAuthor: Ada\n\nINT. ROOM - DAY\n");
        assert!(ensure_fountain_drawing_link(
            &mut document,
            "small.fountain.ink.ron"
        ));
        assert_eq!(
            document.to_text(),
            "Title: Small Things\nAuthor: Ada\nDrawings: small.fountain.ink.ron\n\nINT. ROOM - DAY\n"
        );
        assert_eq!(
            fountain_drawing_link(&document).as_deref(),
            Some("small.fountain.ink.ron")
        );
    }

    #[test]
    fn pen_viewport_position_maps_to_the_rendered_paper() {
        let viewport =
            drawing_viewport_to_ui(Vec2::new(280.0, 385.0), 1.4).expect("valid UI scale");
        let mapped = drawing_viewport_to_document(viewport, Vec2::new(50.0, 125.0), 0.75);
        assert_eq!(mapped, Some(Vec2::new(200.0, 200.0)));
    }

    #[test]
    fn pen_viewport_position_outside_the_paper_uses_negative_coordinates() {
        let viewport = drawing_viewport_to_ui(Vec2::new(68.6, 175.0), 1.4).expect("valid UI scale");
        assert_eq!(
            drawing_viewport_to_document(viewport, Vec2::new(50.0, 125.0), 1.0),
            Some(Vec2::new(-1.0, 0.0))
        );
        assert_eq!(
            drawing_viewport_to_document(Vec2::new(50.0, 125.0), Vec2::new(50.0, 125.0), 0.0),
            None
        );
        assert_eq!(drawing_viewport_to_ui(Vec2::ZERO, 0.0), None);
    }

    #[test]
    fn drawing_camera_viewport_clips_to_the_window() {
        let viewport = drawing_camera_viewport(
            Vec2::new(-10.0, 20.0),
            Vec2::new(300.0, 200.0),
            UVec2::new(250, 150),
        )
        .expect("visible panel");
        assert_eq!(viewport.physical_position, UVec2::new(0, 20));
        assert_eq!(viewport.physical_size, UVec2::new(250, 130));
        assert!(
            drawing_camera_viewport(
                Vec2::new(300.0, 200.0),
                Vec2::new(50.0, 50.0),
                UVec2::new(250, 150),
            )
            .is_none()
        );
    }

    #[test]
    fn vector_view_transform_keeps_the_paper_origin_and_zoom() {
        let viewport = Viewport {
            physical_position: UVec2::new(100, 50),
            physical_size: UVec2::new(700, 500),
            ..default()
        };
        let transform =
            drawing_vector_view_transform(Vec2::new(128.0, 92.0), &viewport, 1.0, 0.75, 1.4)
                .expect("valid drawing transform");

        assert_eq!(transform.translation, Vec3::new(-322.0, 208.0, 0.0));
        assert_eq!(transform.scale, Vec3::new(1.05, 1.05, 1.0));
    }

    #[test]
    fn scaled_pen_contact_persists_a_pressure_sensitive_stroke_to_ron() {
        let mut app = App::new();
        app.add_message::<PenInput>().add_systems(
            Update,
            (handle_drawing_pen_input, save_drawing_sidecar).chain(),
        );

        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        let mut editor = EditorState::from_world(app.world_mut());
        editor.drawing_mode_enabled = true;
        editor.document_format = DocumentFormat::Markdown;
        editor.paths.load_path = PathBuf::from("pen-test.md");
        app.insert_resource(editor);

        let mut document = StrokeDocument::new("pen-test");
        let (canvas, layer) = ensure_drawing_surface(&mut document);
        let sidecar_path = std::env::temp_dir().join(format!(
            "basscript-scaled-pen-{}-{}.ink.ron",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let allowed_viewport = Rect::from_corners(Vec2::new(0.0, 0.0), Vec2::new(900.0, 500.0));
        let mut blocker = VectorStrokeInputBlocker::default();
        blocker.set_regions(blocking_regions_around(
            allowed_viewport,
            Vec2::new(900.0, 500.0),
        ));
        app.insert_resource(document)
            .insert_resource(UiScale(1.4))
            .insert_resource(VectorStrokeSettings::default())
            .insert_resource(blocker)
            .insert_resource(DrawingPenState::default())
            .insert_resource(DrawingSession {
                source_path: Some(PathBuf::from("pen-test.md")),
                sidecar_path: Some(sidecar_path.clone()),
                canvas: Some(canvas),
                layer: Some(layer),
                allowed_viewport: Some(allowed_viewport),
                surface_origin_viewport: Some(Vec2::new(50.0, 125.0)),
                surface_zoom: 0.5,
                surface_extent: CanvasExtent::new(600.0, 900.0),
                save_requested: true,
                ..default()
            });

        let pen = |position| PenInfo {
            window,
            device: PenId::Device(7),
            primary: true,
            position: Some(position),
            tool: PenToolKind::Pen,
        };
        let data = PenData {
            pressure: Some(PenPressure::Normalized(0.4)),
            tilt: Some(InputPenTilt { x: 12, y: -8 }),
            twist: Some(42),
            ..default()
        };

        app.world_mut().write_message(PenInput {
            pen: pen(Vec2::new(210.0, 315.0)),
            action: PenAction::Button {
                button: PenButton::Contact,
                state: ButtonState::Pressed,
                data: data.clone(),
            },
        });
        app.update();
        app.world_mut().write_message(PenInput {
            pen: pen(Vec2::new(280.0, 350.0)),
            action: PenAction::Moved(data.clone()),
        });
        app.update();
        app.world_mut().write_message(PenInput {
            pen: pen(Vec2::new(280.0, 350.0)),
            action: PenAction::Button {
                button: PenButton::Contact,
                state: ButtonState::Released,
                data: data.clone(),
            },
        });
        app.update();

        app.world_mut().write_message(PenInput {
            pen: pen(Vec2::new(35.0, 140.0)),
            action: PenAction::Button {
                button: PenButton::Contact,
                state: ButtonState::Pressed,
                data: data.clone(),
            },
        });
        app.update();
        app.world_mut().write_message(PenInput {
            pen: pen(Vec2::new(42.0, 147.0)),
            action: PenAction::Button {
                button: PenButton::Contact,
                state: ButtonState::Released,
                data,
            },
        });
        app.update();

        let document = app.world().resource::<StrokeDocument>();
        let strokes = &document.canvas(canvas).expect("drawing canvas").layers[0].strokes;
        assert_eq!(strokes.len(), 2);
        assert_eq!(strokes[0].points[0].position(), Vec2::new(200.0, 200.0));
        assert_eq!(strokes[0].points[0].pressure, 0.4);
        assert_eq!(
            strokes[0].points[0].tilt,
            Some(PenTilt { x: 12.0, y: -8.0 })
        );
        assert_eq!(strokes[0].points[0].twist, Some(42.0));
        assert!(
            strokes[0]
                .points
                .last()
                .is_some_and(|point| point.position() == Vec2::new(300.0, 250.0))
        );
        assert_eq!(strokes[1].points[0].position(), Vec2::new(-50.0, -50.0));
        assert!(
            strokes[1]
                .points
                .last()
                .is_some_and(|point| point.position() == Vec2::new(-40.0, -40.0))
        );

        let persisted =
            load_ron_file(&sidecar_path, DocumentLimits::default()).expect("saved drawing RON");
        let persisted_strokes = &persisted
            .canvas(canvas)
            .expect("persisted drawing canvas")
            .layers[0]
            .strokes;
        assert_eq!(persisted_strokes, strokes);
        let _ = std::fs::remove_file(sidecar_path);
    }
}
