use std::marker::PhantomData;

use bytemuck::Pod;

pub struct Buffer<T: Copy + Pod> {
    pub buf: wgpu::Buffer,
    len: usize,
    phantom: PhantomData<T>,
}

impl<T: Copy + Pod> Buffer<T> {
    pub fn new(device: &wgpu::Device, usages: wgpu::BufferUsages, data: &[T]) -> Self {
        use wgpu::util::DeviceExt;
        let contents = bytemuck::cast_slice(data);
        let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents,
            usage: usages | wgpu::BufferUsages::COPY_DST,
            label: None,
        });

        Self {
            buf,
            len: data.len(),
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn buf(&self) -> &wgpu::Buffer {
        &self.buf
    }
}

pub struct DynamicBuffer<T: Copy + Pod>(Buffer<T>);

impl<T: Copy + Pod> DynamicBuffer<T> {
    pub fn new(device: &wgpu::Device, len: usize, usages: wgpu::BufferUsages) -> Self {
        let buffer = Buffer {
            buf: device.create_buffer(&wgpu::BufferDescriptor {
                mapped_at_creation: false,
                size: (std::mem::size_of::<T>() * len) as u64,
                usage: usages | wgpu::BufferUsages::COPY_DST,
                label: None,
            }),
            phantom: PhantomData,
            len,
        };

        Self(buffer)
    }

    pub fn update(&self, queue: &wgpu::Queue, data: &[T], offset: u64) {
        queue.write_buffer(&self.0.buf, offset, bytemuck::cast_slice(data));
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn buf_mut(&mut self) -> &Buffer<T> {
        &self.0
    }

    pub fn buf(&self) -> &Buffer<T> {
        &self.0
    }
}
