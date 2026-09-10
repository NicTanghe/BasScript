use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_resource::{CachedPipeline, PipelineCache, PipelineDescriptor},
    },
    shader::Shader,
    window::{WindowBackendScaleFactorChanged, WindowResized},
    winit::{UpdateMode, WinitSettings},
};

const ACTIVE_INTERVAL: Duration = Duration::from_millis(16);
const FOCUSED_IDLE_INTERVAL: Duration = Duration::from_millis(500);
const UNFOCUSED_IDLE_INTERVAL: Duration = Duration::from_secs(1);
// Give layout, font atlases and the pipelined renderer time to settle after changes.
const SETTLE_TIME: Duration = Duration::from_millis(250);
const PIPELINE_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_PIPELINE_SETTLE: Duration = Duration::from_secs(4);

/// Keep the native event loop ticking during startup and asynchronous rendering work,
/// then sleep between input events and the editor's caret/background-task ticks.
pub(crate) struct ReactiveRenderingPlugin;

impl Plugin for ReactiveRenderingPlugin {
    fn build(&self, app: &mut App) {
        let pipelines = PendingPipelines::default();
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            pipelines.0.store(true, Ordering::Relaxed);
            render_app
                .insert_resource(pipelines.clone())
                .add_systems(Render, track_pending_pipelines.after(RenderSystems::Render));
        }

        // Start reactive, including before Startup runs. Redraw messages written by
        // an Update system cannot help the event loop initialize the renderer.
        app.insert_resource(reactive_settings(true))
            .insert_resource(pipelines)
            .init_resource::<RedrawActivity>()
            .add_systems(Last, update_reactive_rendering);

        if std::env::args().any(|arg| arg == "--diagnose-idle") {
            app.insert_resource(IdleDiagnostics::default())
                .add_systems(Last, report_idle_diagnostics.after(update_reactive_rendering));
        }
    }
}

#[derive(Resource)]
struct IdleDiagnostics {
    since: Instant,
    updates: usize,
    asset_events: usize,
    redraw_requests: usize,
}

impl Default for IdleDiagnostics {
    fn default() -> Self {
        Self {
            since: Instant::now(),
            updates: 0,
            asset_events: 0,
            redraw_requests: 0,
        }
    }
}

fn report_idle_diagnostics(
    mut diagnostic: ResMut<IdleDiagnostics>,
    pending: Res<PendingPipelines>,
    settings: Res<WinitSettings>,
    windows: Query<&Window>,
    changed_nodes: Query<(), Changed<Node>>,
    changed_text: Query<(), Or<(Changed<Text>, Changed<TextSpan>)>>,
    changed_fonts: Query<(), Changed<TextFont>>,
    mut fonts: MessageReader<AssetEvent<Font>>,
    mut images: MessageReader<AssetEvent<Image>>,
    mut redraws: MessageReader<bevy::window::RequestRedraw>,
) {
    diagnostic.updates += 1;
    diagnostic.asset_events += fonts.read().count() + images.read().count();
    diagnostic.redraw_requests += redraws.read().count();
    let elapsed = diagnostic.since.elapsed();
    if elapsed < Duration::from_secs(5) {
        return;
    }
    let focused = windows.iter().any(|window| window.focused);
    info!(
        "[idle] {:.1} updates/s; focused={focused}; mode={:?}; pipelines_pending={}; asset_events={}; redraw_requests={}; changed_nodes={}; changed_text={}; changed_fonts={}",
        diagnostic.updates as f64 / elapsed.as_secs_f64(),
        settings.update_mode(focused),
        pending.0.load(Ordering::Relaxed),
        diagnostic.asset_events,
        diagnostic.redraw_requests,
        changed_nodes.iter().count(),
        changed_text.iter().count(),
        changed_fonts.iter().count(),
    );
    *diagnostic = IdleDiagnostics::default();
}

#[derive(Resource, Clone, Default)]
struct PendingPipelines(Arc<AtomicBool>);

fn is_actionable_pipeline(pipeline: &CachedPipeline) -> bool {
    match &pipeline.descriptor {
        PipelineDescriptor::RenderPipelineDescriptor(desc) => {
            if desc.vertex.shader.id() == AssetId::default() {
                return false;
            }
            if let Some(fragment) = &desc.fragment {
                if fragment.shader.id() == AssetId::default() {
                    return false;
                }
            }
            true
        }
        PipelineDescriptor::ComputePipelineDescriptor(desc) => {
            // Filter out Bevy 0.19's invalid sparse buffer update pipeline on GL/WebGL2
            if desc.label.as_deref() == Some("sparse buffer update pipeline") {
                return false;
            }
            if desc.shader.id() == AssetId::default() {
                return false;
            }
            true
        }
    }
}

#[derive(Default)]
struct PipelineTrackerState {
    first_seen: HashMap<usize, Instant>,
    diagnose_idle: bool,
    last_diagnostic: Option<Instant>,
}

fn track_pending_pipelines(
    cache: Res<PipelineCache>,
    pending: Res<PendingPipelines>,
    mut state: Local<PipelineTrackerState>,
) {
    let now = Instant::now();
    let waiting: HashSet<usize> = cache.waiting_pipelines().collect();

    // Clean up pipelines that are no longer waiting
    state.first_seen.retain(|id, _| waiting.contains(id));

    let mut any_pending = false;
    for (index, pipeline) in cache.pipelines().enumerate() {
        if waiting.contains(&index) && is_actionable_pipeline(pipeline) {
            let first_seen = *state.first_seen.entry(index).or_insert(now);
            if now.duration_since(first_seen) < PIPELINE_TIMEOUT {
                any_pending = true;
            }
        }
    }

    pending.0.store(any_pending, Ordering::Relaxed);

    if state.last_diagnostic.is_none() {
        state.diagnose_idle = std::env::args().any(|arg| arg == "--diagnose-idle");
        state.last_diagnostic = Some(now);
    }
    if state.diagnose_idle
        && state
            .last_diagnostic
            .is_some_and(|t| t.elapsed() >= Duration::from_secs(5))
    {
        info!(
            "[idle] render pipeline tracker ran; waiting={waiting:?}; actionable_pending={any_pending}"
        );
        for (index, pipeline) in cache.pipelines().enumerate() {
            if waiting.contains(&index) {
                info!(
                    "[idle] pipeline {index}: {:?}; descriptor={:?}",
                    pipeline.state, pipeline.descriptor
                );
            }
        }
        state.last_diagnostic = Some(now);
    }
}

#[derive(Resource)]
struct RedrawActivity {
    settle_until: Duration,
    pipeline_settle_start: Option<Duration>,
}

impl Default for RedrawActivity {
    fn default() -> Self {
        Self {
            settle_until: SETTLE_TIME,
            pipeline_settle_start: None,
        }
    }
}

impl RedrawActivity {
    fn update(&mut self, now: Duration, changed: bool, pipelines_pending: bool) -> bool {
        if changed {
            self.settle_until = now.saturating_add(SETTLE_TIME);
            self.pipeline_settle_start = None;
        } else if pipelines_pending {
            let start = *self.pipeline_settle_start.get_or_insert(now);
            if now.saturating_sub(start) < MAX_PIPELINE_SETTLE {
                self.settle_until = now.saturating_add(SETTLE_TIME);
            }
        } else {
            self.pipeline_settle_start = None;
        }
        now < self.settle_until
    }
}

fn reactive_settings(active: bool) -> WinitSettings {
    WinitSettings {
        focused_mode: UpdateMode::reactive(if active {
            ACTIVE_INTERVAL
        } else {
            FOCUSED_IDLE_INTERVAL
        }),
        unfocused_mode: UpdateMode::reactive_low_power(if active {
            ACTIVE_INTERVAL
        } else {
            UNFOCUSED_IDLE_INTERVAL
        }),
    }
}

#[derive(SystemParam)]
struct RedrawEvents<'w, 's> {
    fonts: MessageReader<'w, 's, AssetEvent<Font>>,
    images: MessageReader<'w, 's, AssetEvent<Image>>,
    shaders: MessageReader<'w, 's, AssetEvent<Shader>>,
    resized: MessageReader<'w, 's, WindowResized>,
    scaled: MessageReader<'w, 's, WindowBackendScaleFactorChanged>,
}

impl RedrawEvents<'_, '_> {
    fn read_changes(&mut self) -> bool {
        // Drain every reader, even if an earlier one already contains changes.
        (self.fonts.read().count() > 0)
            | (self.images.read().count() > 0)
            | (self.shaders.read().count() > 0)
            | (self.resized.read().count() > 0)
            | (self.scaled.read().count() > 0)
    }
}

fn wall_clock_elapsed(time: &Time<Real>) -> Duration {
    // With pipelined rendering, Time<Real> may still contain the previous render's
    // timestamp. Include the idle gap so a fresh settling window cannot expire
    // immediately when that timestamp catches up on the next frame.
    time.elapsed().saturating_add(
        time.last_update()
            .map_or(Duration::ZERO, |last_update| last_update.elapsed()),
    )
}

fn update_reactive_rendering(
    time: Res<Time<Real>>,
    pending: Res<PendingPipelines>,
    mut activity: ResMut<RedrawActivity>,
    mut settings: ResMut<WinitSettings>,
    mut events: RedrawEvents,
) {
    let active = activity.update(
        wall_clock_elapsed(&time),
        events.read_changes(),
        pending.0.load(Ordering::Relaxed),
    );
    let next = reactive_settings(active);
    if settings.focused_mode != next.focused_mode || settings.unfocused_mode != next.unfocused_mode
    {
        debug!(
            "Reactive rendering: {}",
            if active { "settling" } else { "idle" }
        );
        *settings = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_and_unfocused_modes_are_always_reactive() {
        for active in [false, true] {
            let settings = reactive_settings(active);
            for mode in [settings.focused_mode, settings.unfocused_mode] {
                assert!(matches!(mode, UpdateMode::Reactive { wait, .. } if !wait.is_zero()));
            }
        }
    }

    #[test]
    fn startup_settles_to_idle_without_a_redraw_feedback_loop() {
        let mut activity = RedrawActivity::default();
        assert!(activity.update(Duration::ZERO, false, false));
        assert!(!activity.update(SETTLE_TIME, false, false));
        assert!(!activity.update(Duration::from_secs(10), false, false));
    }

    #[test]
    fn settling_includes_time_spent_idle_since_the_last_render() {
        let mut time = Time::<Real>::default();
        let previous_render = bevy::platform::time::Instant::now() - UNFOCUSED_IDLE_INTERVAL;
        time.update_with_instant(previous_render);
        assert_eq!(time.elapsed(), Duration::ZERO);
        assert!(wall_clock_elapsed(&time) >= UNFOCUSED_IDLE_INTERVAL);

        let mut activity = RedrawActivity::default();
        assert!(activity.update(wall_clock_elapsed(&time), true, false));
        time.update_with_instant(bevy::platform::time::Instant::now());
        assert!(activity.update(wall_clock_elapsed(&time), false, false));
    }

    #[test]
    fn slow_pipelines_are_not_limited_to_thirty_startup_frames() {
        let mut activity = RedrawActivity::default();
        for frame in 0..120 {
            assert!(activity.update(ACTIVE_INTERVAL * frame, false, true));
        }
        let finished = ACTIVE_INTERVAL * 120;
        assert!(activity.update(finished, false, false));
        assert!(!activity.update(finished + SETTLE_TIME, false, false));
    }

    #[test]
    fn late_assets_and_resizes_resume_then_settle() {
        let mut activity = RedrawActivity::default();
        let now = Duration::from_secs(5);
        assert!(!activity.update(now, false, false));
        assert!(activity.update(now, true, false));
        assert!(activity.update(now + ACTIVE_INTERVAL, false, false));
        assert!(!activity.update(now + SETTLE_TIME, false, false));
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time<Real>>()
            .add_message::<AssetEvent<Font>>()
            .add_message::<AssetEvent<Image>>()
            .add_message::<AssetEvent<Shader>>()
            .add_message::<WindowResized>()
            .add_message::<WindowBackendScaleFactorChanged>()
            .add_plugins(ReactiveRenderingPlugin);
        app
    }

    fn advance(app: &mut App, delta: Duration) {
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(delta);
        app.update();
    }

    fn assert_activity(app: &App, active: bool) {
        let actual = app.world().resource::<WinitSettings>();
        let expected = reactive_settings(active);
        assert_eq!(actual.focused_mode, expected.focused_mode);
        assert_eq!(actual.unfocused_mode, expected.unfocused_mode);
    }

    #[test]
    fn event_readers_drain_together_and_idle_stays_idle() {
        let mut app = test_app();
        assert_activity(&app, true);
        advance(&mut app, SETTLE_TIME);
        assert_activity(&app, false);

        app.world_mut().write_message(AssetEvent::Added {
            id: Handle::<Font>::default().id(),
        });
        app.world_mut().write_message(AssetEvent::Modified {
            id: Handle::<Image>::default().id(),
        });
        app.world_mut().write_message(AssetEvent::Added {
            id: Handle::<Shader>::default().id(),
        });
        app.world_mut().write_message(WindowResized {
            window: Entity::PLACEHOLDER,
            width: 800.0,
            height: 600.0,
        });
        app.world_mut()
            .write_message(WindowBackendScaleFactorChanged {
                window: Entity::PLACEHOLDER,
                scale_factor: 1.5,
            });
        advance(&mut app, ACTIVE_INTERVAL);
        assert_activity(&app, true);

        advance(&mut app, SETTLE_TIME);
        assert_activity(&app, false);
        advance(&mut app, Duration::from_secs(1));
        assert_activity(&app, false);
    }

    #[test]
    fn render_thread_work_keeps_reactive_ticks_until_it_finishes() {
        let mut app = test_app();
        let pending = app.world().resource::<PendingPipelines>().clone();
        pending.0.store(true, Ordering::Relaxed);
        advance(&mut app, Duration::from_secs(2));
        assert_activity(&app, true);

        pending.0.store(false, Ordering::Relaxed);
        advance(&mut app, SETTLE_TIME);
        assert_activity(&app, false);
    }

    #[test]
    fn pipeline_settle_is_capped_when_no_user_changes() {
        let mut activity = RedrawActivity::default();
        let step = Duration::from_millis(500);
        let mut now = Duration::ZERO;
        while now < MAX_PIPELINE_SETTLE {
            assert!(activity.update(now, false, true));
            now += step;
        }
        // Once past MAX_PIPELINE_SETTLE + SETTLE_TIME without user changes, it must settle to idle
        assert!(!activity.update(now + SETTLE_TIME, false, true));
    }

    #[test]
    fn unactionable_sparse_buffer_pipeline_is_filtered() {
        use bevy::material::descriptor::ComputePipelineDescriptor;
        use bevy::render::render_resource::CachedPipelineState;

        let dummy_pipeline = CachedPipeline {
            descriptor: PipelineDescriptor::ComputePipelineDescriptor(Box::new(
                ComputePipelineDescriptor {
                    label: Some("sparse buffer update pipeline".into()),
                    shader: Handle::default(),
                    ..default()
                },
            )),
            state: CachedPipelineState::Queued,
        };
        assert!(!is_actionable_pipeline(&dummy_pipeline));

        let empty_shader_pipeline = CachedPipeline {
            descriptor: PipelineDescriptor::ComputePipelineDescriptor(Box::new(
                ComputePipelineDescriptor {
                    label: Some("custom compute".into()),
                    shader: Handle::default(),
                    ..default()
                },
            )),
            state: CachedPipelineState::Queued,
        };
        assert!(!is_actionable_pipeline(&empty_shader_pipeline));
    }
}
