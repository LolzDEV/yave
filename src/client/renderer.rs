use pollster::block_on;
use thunderdome::Arena;
use wgpu::{Backends, Features, Instance, Limits, PowerPreference};
use winit::window::Window;

/// Game renderer (wgpu)
pub struct Renderer {
    /// The surface where the game is rendered on
    pub surface: wgpu::Surface,
    /// The render pipeline layout which handles voxel rendering
    pub world_pipeline_layout: wgpu::PipelineLayout,
    /// The bridge between the GPU and the renderer
    pub device: wgpu::Device,
    /// A queue used to submit commands to the device
    pub queue: wgpu::Queue,
    /// An arena which holds buffers
    pub arena: Arena<wgpu::Buffer>
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        })).unwrap();

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Device"),
            features: Features::empty(),
            limits: Limits::default(),
        }, None)).unwrap();

        let arena = Arena::new();

        let world_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("World Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });

        Self {
            surface,
            device,
            world_pipeline_layout,
            queue,
            arena,
        }
    }
}