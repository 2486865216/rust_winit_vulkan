use std::sync::Arc;
use image::{ImageBuffer, Rgba};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::CopyImageToBufferInfo;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::MemoryUsage;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use crate::example::buffer::{choose_device, create_auto_command_buffer_builder, create_buffer_allocator, create_device, create_instance, create_iter_buffer, create_memory_allocator, get_queue};
use crate::example::image_glsl::shader;

pub fn operator_image_shader() {
    //创建一个实例
    let instance = create_instance();
    //选择一个物理设备
    let physical_device = choose_device(instance.clone());
    //获取队列
    let queue_index = get_queue(physical_device.clone());
    //创建设备
    let (device, mut queues) = create_device(physical_device.clone(), queue_index);
    let queue = queues.next().unwrap();
    //内存分配器
    let memory_allocator = create_memory_allocator(device.clone());

    //创建图像
    let image = StorageImage::new(
        &memory_allocator,
        ImageDimensions::Dim2d {
            width: 1024,
            height: 1024,
            array_layers: 1,
        },
        Format::R8G8B8A8_UNORM,
        Some(queue.queue_family_index()),
    ).unwrap();

    //创建一个着色器
    let view = ImageView::new_default(image.clone()).unwrap();

    //创建描述符集
    let shader = shader::load(device.clone()).expect("failed to create shader module");
    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        shader.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    ).expect("failed to create compute pipeline");

    let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout.clone(),
        [WriteDescriptorSet::image_view(0, view.clone())],
    ).unwrap();

    //创建一个缓冲区来存储图像输出
    let buffer = create_iter_buffer(
        Arc::new(memory_allocator),
        BufferUsage::TRANSFER_DST,
        MemoryUsage::Download,
        (0..1024 * 1024 * 4).map(|_| 0u8)
    );

    //创建命令缓冲区
    let command_buffer_allocator = create_buffer_allocator(device.clone());
    let mut builder = create_auto_command_buffer_builder(command_buffer_allocator, queue_index);

    builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0,
             set
        )
        .dispatch([1024 / 8, 1024 / 8, 1])
        .unwrap()
        .copy_image_to_buffer(
            CopyImageToBufferInfo::image_buffer(
                image.clone(),
                buffer.clone()
            )
        ).unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let content = buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &content[..]).unwrap();
    image.save("image_shader.png").unwrap();
    println!("Everything is succeeded!");
}