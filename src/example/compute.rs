//出于本指南的目的，我们将做一些非常简单的事情：我们将乘以 65536 值由常量 12.尽管这没有任何用处，但
//这是一个很好的开始 点示例。GPU 的大多数实际用途都涉及复杂的数学算法，因此 不太适合教程。

use std::sync::Arc;
//如上所述，您不需要使用任何循环或类似的东西。我们所有人 要做的是写一个值上执行的操作，然后要求GPU执行 它65536次。
use crate::example::buffer::*;
use vulkano::buffer::BufferUsage;
use vulkano::memory::allocator::MemoryUsage;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::sync;
use vulkano::sync::GpuFuture;

use crate::example::glsl::*;

pub fn operator_computer() {
    let data_iter = 0..65536u32;

    let instance = create_instance();
    let physical_device = choose_device(instance.clone());
    let queue_index = get_queue(physical_device.clone());
    let (device, mut queues) = create_device(physical_device.clone(), queue_index);
    let memory_allocator = create_memory_allocator(device.clone());

    let data_buffer = create_iter_buffer(Arc::new(memory_allocator), BufferUsage::STORAGE_BUFFER, MemoryUsage::Upload, data_iter);

    let shader = cs::load(device.clone()).expect("failed to create shader module");
    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        shader.entry_point("main").unwrap(),
        &(),
        None,
        |_| {}
    ).expect("failed to create compute pipeline");

    //创建描述符集
    //就像缓冲区和命令缓冲区一样，我们也需要一个描述符集的分配器。
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let pipeline_layout = compute_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();

    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts.get(descriptor_set_layout_index).unwrap();
    let descriptor_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        descriptor_set_layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())]
    ).unwrap();

    //创建命令缓冲区
    let command_buffer_allocator = create_buffer_allocator(device.clone());
    let mut command_buffer_builder = create_auto_command_buffer_builder(command_buffer_allocator, queue_index);

    let work_group_counts = [1024, 1, 1];
    command_buffer_builder.
        bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            descriptor_set_layout_index as u32,
            descriptor_set
        )
        .dispatch(work_group_counts).unwrap();

    let command_buffer = command_buffer_builder.build().unwrap();

    //提交命令缓冲区
    let future = sync::now(device.clone())
        .then_execute(queues.next().unwrap(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    //等待它完成
    future.wait(None).unwrap();

    //完成后，我们可以检查管道是否已正确执行
    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }
    println!("Everything is succeeded!");
}