use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo,
    },
    device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

pub fn operator_buffer() {
    //创建一个实例，如果系统不可使用vulkan，报错
    let library = VulkanLibrary::new().expect("no local Vulkan libray");
    let instance =
        Instance::new(library, InstanceCreateInfo::default()).expect("failed to create instance");

    //选择一个设备，一台计算机可能有多个设备能够使用vulkan
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("cloud not enumerate devices")
        .next()
        .expect("not devices available");

    //获取队列
    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_queue_family_index, queue_family_properties)| {
            queue_family_properties
                .queue_flags
                .contains(QueueFlags::GRAPHICS)
        })
        .expect("cloud not find graphical queue family") as u32;

    //创建设备 创建设备返回两件事：设备本身，以及队列对象列表 稍后将允许我们提交操作。
    let (devices, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    //创建内存分配器
    let device_clone = devices.clone();
    let memory_allocator = StandardMemoryAllocator::new_default(device_clone);

    //第一步是创建两个 CPU 可访问的缓冲区：源缓冲区和目标缓冲区
    let source_content: Vec<i32> = (0..64).collect();
    let source = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        source_content,
    )
    .expect("failed to create source buffer");

    let destination_content: Vec<i32> = (0..64).map(|_| 0).collect();
    let destination = Buffer::from_iter(
        &memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            usage: MemoryUsage::Download,
            ..Default::default()
        },
        destination_content,
    )
    .expect("failed to create destination buffer");

    //创建命令缓冲区分配器
    //就像缓冲区一样，您需要一个分配器来分配多个命令缓冲区，但不能使用内存分配器。您必须使用命令缓冲区分配器
    let device_clone_one = devices.clone();
    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device_clone_one,
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    //我们将命令提交到 GPU，因此让我们创建一个主命令缓冲区
    let mut builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue_family_index,
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
        .unwrap();

    let command_buffer = builder.build().unwrap();

    //提交和同步
    //最后一步是实际发送命令缓冲区并在 GPU 中执行它。我们可以通过以下方式做到这一点 与 GPU 同步，然后执行命令缓冲区：
    //为了阅读的内容并确保我们的副本成功，我们需要 等待操作完成。为此，我们需要对 GPU 进行编程以发回一个特殊的 信号，让我们知道它已经结束了。这种信号被称为围栏，它让 我们知道GPU何时达到某个执行点
    let future = sync::now(devices.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();
    //只有在完成此操作后，我们才能安全地调用并检查我们的复制是否成功
    let src_content = source.read().unwrap();
    let destination_content = destination.read().unwrap();

    assert_eq!(&*src_content, &*destination_content);

    println!("Everything succeed!");
}

pub fn create_data_buffer(data: T) -> i32 {
    0
}
