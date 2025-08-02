use rand::Rng;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState, shell::{
        wlr_layer::{Anchor, LayerShell, LayerSurface}, WaylandSurface
    }
};
use wgpu::{util::DeviceExt, BindGroup, Buffer};
use std::{borrow::Cow, ptr::NonNull, time::Instant};
use wayland_client::{
    globals::registry_queue_init, Connection, Proxy, QueueHandle
};

mod implementations;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}
impl Vertex {
    #[allow(dead_code)]
    fn triangle(x: f32, y: f32, w: f32, h: f32) -> [Self; 3] {
        let x0 = x;
        let x1 = x + w;
        let y0 = y;
        let y1 = y + h;

        [
            Vertex { position: [x0, y0] },
            Vertex { position: [x1, y0] },
            Vertex { position: [x1, y1] },
        ]
    }
    #[allow(dead_code)]
    fn rectangle(x: f32, y: f32, w: f32, h: f32) -> [Self; 6] {
        let x0 = x;
        let x1 = x + w;
        let y0 = y;
        let y1 = y + h;

        [
            Vertex { position: [x0, y0] }, // Triangle 1
            Vertex { position: [x1, y0] },
            Vertex { position: [x1, y1] },
            Vertex { position: [x0, y0] }, // Triangle 2
            Vertex { position: [x1, y1] },
            Vertex { position: [x0, y1] },
        ]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    direction: [f32; 2],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
}

impl Uniforms {
    fn new() -> Self {
        Self { time: 0.0 }
    }
}

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    // Initialize xdg_shell handlers so we can select the correct adapter
    let compositor_state =
        CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_state = LayerShell::bind(&globals, &qh).expect("layer_shell not available");

    let surface = compositor_state.create_surface(&qh);
    // Create the window for adapter selection
    let layer = layer_state.create_layer_surface(&qh, surface, smithay_client_toolkit::shell::wlr_layer::Layer::Top, Some(""), None);
    layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    let (width, height) = (400, 400);
    layer.set_size(0, 0); // 0 width = stretch to full width
    layer.set_opaque_region(None);
    layer.commit();


    // Initialize wgpu
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Create the raw window handle for the surface.
    let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
        NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
    ));
    let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
        NonNull::new(layer.wl_surface().id().as_ptr() as *mut _).unwrap(),
    ));

    let surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
            .unwrap()
    };

    // Pick a supported adapter
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("Failed to find suitable adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
        .expect("Failed to request device");


    let mut surface_config = surface.get_default_config(&adapter, 200, 200).unwrap();
    surface_config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
    surface_config.format = wgpu::TextureFormat::Bgra8Unorm;

    let (layout, group, uniform_buffer, uniforms) = create_uniforms(&device);
    let (vertex_buffer, instance_buffer, vertex_count, instance_count) = create_vertex_buffer(&device, width as f32, height as f32);

    let render_pipeline = create_pipeline(&device, surface_config.format, &layout);

    let mut wgpu = Wgpu {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),

        start_time: Instant::now(),
        first_configure: true,
        exit: false,
        width: 256,
        height: 256,
        window: layer,
        device,
        surface,
        queue,
        render_pipeline,
        group,

        uniforms,
        uniform_buffer,
        vertex_buffer,
        instance_buffer,
        vertex_count,
        instance_count,
    };

    // We don't draw immediately, the configure will notify us when to first draw.
    loop {
        event_queue.blocking_dispatch(&mut wgpu).unwrap();

        if wgpu.exit {
            break;
        }
    }

    // On exit we must destroy the surface before the window is destroyed.
    drop(wgpu.surface);
    drop(wgpu.window);
}
struct Wgpu {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,

    start_time: Instant,
    exit: bool,
    first_configure: bool,
    width: u32,
    height: u32,
    window: LayerSurface,

    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    render_pipeline: wgpu::RenderPipeline,
    group: wgpu::BindGroup,

    uniforms: Uniforms,
    uniform_buffer: Buffer,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    vertex_count: u32,
    instance_count: u32,

}

impl Wgpu {
    fn draw(&mut self, _qh: &QueueHandle<Self>) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.update_time(elapsed);
        if elapsed > 2.5  {
            std::process::exit(0);
        }
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_bind_group(0, &self.group, &[]);
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.draw(0..self.vertex_count, 0..self.instance_count);
        }
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

    }
    pub fn update_time(&mut self, time: f32) {
        self.uniforms.time = time;
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&self.uniforms),
        );
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Load theushaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });
    let vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    };
    let instance_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2, // v_start
                offset: 0,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3, // color
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 2,
            },
        ],
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex_buffer_layout, instance_buffer_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_chain_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), // ✅ Required
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            // strip_index_format: None,
            // front_face: wgpu::FrontFace::Ccw,
            // cull_mode: Some(wgpu::Face::Back),
            // // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // // or Features::POLYGON_MODE_POINT
            // polygon_mode: wgpu::PolygonMode::Fill,
            // // Requires Features::DEPTH_CLIP_CONTROL
            // unclipped_depth: false,
            // // Requires Features::CONSERVATIVE_RASTERIZATION
            // conservative: false,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn create_uniforms(device: &wgpu::Device) -> (wgpu::BindGroupLayout, BindGroup, Buffer, Uniforms) {
    let uniforms = Uniforms::new();
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("uniform_bind_group_layout"),
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("uniform_bind_group"),
    });

    (uniform_bind_group_layout, uniform_bind_group, uniform_buffer, uniforms)
}

fn create_vertex_buffer(device: &wgpu::Device, width: f32, height: f32) -> (Buffer, Buffer, u32, u32) {
    // Only 1 rectangle vertices here, since instances define position:
    let rectangle = Vertex::rectangle(0.0, 0.0, 0.00002 * height, 0.00002 * width);
    let mut rng = rand::rng();
    let colors: Vec<[f32; 3]> = vec![
        [1.0, 0.0, 0.0],   // red
        [1.0, 0.5, 0.0],   // orange
        [1.0, 1.0, 0.0],   // yellow
        [0.0, 1.0, 0.0],   // green
        [0.0, 1.0, 1.0],   // cyan
        [0.0, 0.0, 1.0],   // blue
        [0.5, 0.0, 1.0],   // purple
        [1.0, 0.0, 1.0],   // magenta
        [1.0, 0.7, 0.7],   // pink
        [0.7, 1.0, 0.7],   // light green
        [0.7, 0.7, 1.0],   // light blue
        [1.0, 0.9, 0.6],   // cream
        [0.8, 0.4, 0.0],   // burnt orange
        [0.4, 0.8, 0.1],   // lime green
        [0.1, 0.4, 0.8],   // steel blue
        [0.8, 0.1, 0.8],   // violet
        [1.0, 0.6, 0.9],   // bright pink
        [0.6, 0.6, 0.6],   // gray
        [0.9, 0.8, 0.1],   // gold
        [0.3, 1.0, 0.7],   // mint green
        [0.2, 0.6, 0.2],   // forest green
        [0.9, 0.3, 0.2],   // brick red
        [0.9, 0.9, 0.9],   // light gray
        [0.3, 0.2, 0.6],   // indigo
        [0.1, 0.8, 0.8],   // turquoise
        [0.9, 0.5, 0.7],   // rose pink
        [0.8, 0.7, 0.3],   // mustard
        [0.5, 0.9, 0.4],   // pea green
        [0.4, 0.7, 0.9],   // sky blue
        [0.7, 0.3, 0.6],   // mauve
        [0.9, 0.9, 0.4],   // lemon yellow
        [0.2, 0.4, 0.3],   // dark teal
        [0.8, 0.2, 0.2],   // cherry red
        [0.6, 0.6, 0.8],   // lavender
        [0.3, 0.8, 0.5],   // seafoam
    ];
    let color_count = colors.len();

    let instances = (0..200).map(|_| {
        let x = rng.random_range(-1.0..1.0) as f32;
        let y_max = (1.0 - x*x).sqrt() * 2.5;
        let y = rng.random_range(-0.5..y_max);
        InstanceData {
            direction: [ x * 1.2, y ],
            color: colors.get(rng.random_range(0..color_count)).unwrap().clone(),
        }
    }).collect::<Vec<InstanceData>>();


    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Rectangle Vertex Buffer"),
        contents: bytemuck::cast_slice(&rectangle),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsages::VERTEX,
    });

    (vertex_buffer, instance_buffer, rectangle.len() as u32, instances.len() as u32)
}
