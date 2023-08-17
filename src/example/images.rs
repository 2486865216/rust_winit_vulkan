use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::format::{ClearColorValue, Format};
use vulkano::command_buffer::{ClearColorImageInfo, CopyImageToBufferInfo};
use vulkano::memory::allocator::MemoryUsage;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use crate::example::buffer::{choose_device, create_auto_command_buffer_builder, create_buffer_allocator, create_device, create_instance, create_iter_buffer, create_memory_allocator, get_queue};
use image::{ImageBuffer, Rgba};

pub fn operator_image() {
    let instance = create_instance();
    let physical_device = choose_device(instance.clone());
    let queue_index = get_queue(physical_device.clone());
    let (device, mut queues) = create_device(physical_device.clone(), queue_index);
    let queue = queues.next().unwrap();
    let memory_allocator = create_memory_allocator(device.clone());
    //映像创建
    //创建图像与创建缓冲区非常相似。就像有多个不同的 Vulkano 中的结构表示缓冲区，还有多个不同的结构表示图像。
    // 在这里，我们将使用一个 StorageImage，这是一个通用映像。
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

    //清除图像
    let command_buffer_allocator = create_buffer_allocator(device.clone());
    let mut builder = create_auto_command_buffer_builder(command_buffer_allocator, queue.queue_family_index());

    /*let x = builder.clear_color_image(
        ClearColorImageInfo {
            clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
            ..ClearColorImageInfo::image(image.clone())
        }
    ).unwrap();*/

    //图像复制到缓冲区
    let buf = create_iter_buffer(Arc::new(memory_allocator),
                                 BufferUsage::TRANSFER_DST,
                                 MemoryUsage::Download,
                                 (0..1024 * 1024 * 4).map(|_| 0u8));
    builder
        .clear_color_image(ClearColorImageInfo {
            clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
            ..ClearColorImageInfo::image(image.clone())
        })
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            image.clone(),
            buf.clone(),
        ))
        .unwrap();

    let command_buffer = builder.build().unwrap();

    //我们不要忘记执行命令缓冲区并阻止，直到操作完成
    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &content[..]).unwrap();

    image.save("image.png").unwrap();
    println!("Everything succeeded!");
}