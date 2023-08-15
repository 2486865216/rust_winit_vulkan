//出于本指南的目的，我们将做一些非常简单的事情：我们将乘以 65536 值由常量 12.尽管这没有任何用处，但
//这是一个很好的开始 点示例。GPU 的大多数实际用途都涉及复杂的数学算法，因此 不太适合教程。

use std::sync::Arc;
//如上所述，您不需要使用任何循环或类似的东西。我们所有人 要做的是写一个值上执行的操作，然后要求GPU执行 它65536次。
use crate::example::buffer::*;
use vulkano::buffer::BufferUsage;
use vulkano::memory::allocator::MemoryUsage;

use crate::example::glsl;
// use crate::example::glsl::cs;

pub fn test() {
    let data_iter = 0..65536u32;

    let instance = create_instance();
    let physical_device = choose_device(instance.clone());
    let queue_index = get_queue(physical_device.clone());
    let (device, queues) = create_device(physical_device.clone(), queue_index);
    let memory_allocator = create_memory_allocator(device.clone());

    let data_buffer = create_iter_buffer(Arc::new(memory_allocator), BufferUsage::STORAGE_BUFFER, MemoryUsage::Upload, data_iter);
}