//! Initialize native OpenGL with Winit's actual display connection.
//!
//! Bevy 0.19's automatic renderer passes `display: None` to wgpu. Wgpu 29
//! requires a display for EGL presentation; otherwise it can pick a headless
//! configuration with no presentation modes. Use Bevy's manual initialization
//! hook at the normal RenderPlugin position, keeping the rest of its renderer.

use std::sync::Arc;

use bevy::{
    prelude::*,
    render::{
        RenderPlugin,
        renderer::{RenderAdapter, RenderAdapterInfo, RenderInstance, RenderQueue, WgpuWrapper},
        settings::RenderCreation,
    },
    tasks::block_on,
    winit::DisplayHandleWrapper,
};

pub(crate) struct LinuxRendererPlugin;

impl Plugin for LinuxRendererPlugin {
    fn build(&self, app: &mut App) {
        // WinitPlugin precedes RenderPlugin and owns this connection for both
        // X11 and Wayland. Wgpu retains the cloned owned display handle.
        let display = app.world().resource::<DisplayHandleWrapper>().0.clone();
        let mut descriptor = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display));
        descriptor.backends = wgpu::Backends::GL;
        descriptor.flags = wgpu::InstanceFlags::default().with_env();
        // Wgpu's indirect-call validator uses a compute shader, unavailable in
        // GL 3.3 contexts. Preserve the other API/shader validation flags.
        descriptor
            .flags
            .remove(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL);
        let instance = wgpu::Instance::new(descriptor);
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::from_env().unwrap_or_default(),
            ..default()
        }))
        .expect("Linux glass requires an OpenGL 3.3 / GLES 3 compatible display driver");
        let info = adapter.get_info();
        info!("[window] Linux glass renderer: {info:?}");

        // The editor's 2D UI does not need compute or storage buffers. Use Bevy's
        // uniform-buffer and CPU preprocessing paths, including on GL 3.3.
        let limits = wgpu::Limits::downlevel_webgl2_defaults()
            .using_resolution(adapter.limits())
            .using_alignment(adapter.limits());
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("BasScript Linux glass"),
            required_features: adapter.features()
                & wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            required_limits: limits,
            ..default()
        }))
        .expect("Failed to initialize the Linux glass renderer");

        app.add_plugins(RenderPlugin {
            render_creation: RenderCreation::manual(
                device.into(),
                RenderQueue(Arc::new(WgpuWrapper::new(queue))),
                RenderAdapterInfo(WgpuWrapper::new(info)),
                RenderAdapter(Arc::new(WgpuWrapper::new(adapter))),
                RenderInstance(Arc::new(WgpuWrapper::new(instance))),
            ),
            ..default()
        });
    }
}
