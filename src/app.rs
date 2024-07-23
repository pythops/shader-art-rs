use crate::pipeline::Pipeline;
use wgpu::MemoryHints;
use winit::window::Window;

#[derive(Clone)]
pub struct TextureSize {
    pub height: u32,
    pub width: u32,
}

pub struct Surface<'a> {
    window: &'a Window,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    pub surface_size: winit::dpi::PhysicalSize<u32>,
}

pub struct App<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub surface: Option<Surface<'a>>,
    pub pipeline: Pipeline,
    pub output_buffer: Option<wgpu::Buffer>,
    pub texture_size: Option<TextureSize>,
}

impl<'a> App<'a> {
    pub async fn new_without_window(speed: u8, resolution: [u16; 2]) -> App<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: Default::default(),
            gles_minor_version: Default::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        let texture_size = TextureSize {
            width: resolution[0].into(),
            height: resolution[1].into(),
        };

        let padded_bytes_per_row = Self::calculate_padding(texture_size.width);

        let output_buffer_size =
            (padded_bytes_per_row * texture_size.height) as wgpu::BufferAddress;

        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };

        let output_buffer = device.create_buffer(&output_buffer_desc);

        let pipeline = Pipeline::new(
            &device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            [texture_size.height as f32, texture_size.width as f32],
            speed,
        );

        Self {
            device,
            queue,
            surface: None,
            pipeline,
            output_buffer: Some(output_buffer),
            texture_size: Some(texture_size),
        }
    }

    pub async fn new_with_window(window: &'a Window, speed: u8) -> App<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: Default::default(),
            gles_minor_version: Default::default(),
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let pipeline = Pipeline::new(
            &device,
            surface_config.format,
            [size.height as f32, size.width as f32],
            speed,
        );

        let app_surface = Surface {
            window,
            surface,
            surface_config,
            surface_size: size,
        };

        Self {
            device,
            queue,
            surface: Some(app_surface),
            pipeline,
            output_buffer: None,
            texture_size: None,
        }
    }

    // https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
    pub fn calculate_padding(size: u32) -> u32 {
        let pixel_size = std::mem::size_of::<[u8; 4]>() as u32;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded_bytes_per_row = pixel_size * size;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        unpadded_bytes_per_row + padding
    }

    pub fn window(&self) -> &Window {
        self.surface.as_ref().unwrap().window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let app_surface = self.surface.as_mut().unwrap();
            app_surface.surface_size = new_size;
            app_surface.surface_config.width = new_size.width;
            app_surface.surface_config.height = new_size.height;
            app_surface
                .surface
                .configure(&self.device, &app_surface.surface_config);
            self.pipeline
                .common
                .update_dimensions([new_size.height as f32, new_size.width as f32]);
            self.queue.write_buffer(
                &self.pipeline.uniform_buffer,
                0,
                bytemuck::cast_slice(&[self.pipeline.common]),
            );
        }
    }

    pub fn update(&mut self) {
        self.pipeline.common.update_time();
        self.queue.write_buffer(
            &self.pipeline.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.pipeline.common]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let app_surface = self.surface.as_ref().unwrap();
        let output = app_surface.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline.render_pipeline);
            render_pass.set_vertex_buffer(0, self.pipeline.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.pipeline.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.set_bind_group(0, &self.pipeline.shader_binding_group, &[]);
            render_pass.draw_indexed(0..self.pipeline.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub async fn run(&mut self, frames: &mut Vec<Vec<u8>>) {
        let texture_size = self.texture_size.as_ref().unwrap();

        let pixel_size = std::mem::size_of::<[u8; 4]>() as u32;
        let unpadded_bytes_per_row = pixel_size * texture_size.width;
        let padded_bytes_per_row = Self::calculate_padding(texture_size.width);

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                height: texture_size.height,
                width: texture_size.width,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };

        let texture = self.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&self.pipeline.render_pipeline);
            render_pass.set_vertex_buffer(0, self.pipeline.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.pipeline.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.set_bind_group(0, &self.pipeline.shader_binding_group, &[]);
            render_pass.draw_indexed(0..self.pipeline.num_indices, 0, 0..1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: self.output_buffer.as_ref().unwrap(),
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(texture_size.width),
                },
            },
            wgpu::Extent3d {
                height: texture_size.height,
                width: texture_size.width,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let output_buffer = self.output_buffer.as_mut().unwrap();

        let buffer_slice = output_buffer.slice(..);

        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);

        match rx.receive().await {
            Some(Ok(())) => {
                let padded_data = buffer_slice.get_mapped_range();
                let data = padded_data
                    .chunks(padded_bytes_per_row as _)
                    .flat_map(|chunk| &chunk[..unpadded_bytes_per_row as _])
                    .copied()
                    .collect::<Vec<_>>();
                drop(padded_data);
                output_buffer.unmap();
                frames.push(data);
            }
            _ => eprintln!("Something went wrong"),
        }
    }
}
