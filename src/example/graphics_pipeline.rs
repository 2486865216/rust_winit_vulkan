//到目前为止，我们已经创建了执行两种操作的命令缓冲区：
//
// 内存传输（在缓冲区和图像之间复制数据，清除图像）。
// 计算操作（调度计算着色器）。
// 虽然这两种操作就足够了，以便将GPU的强大功能用于 并行计算（如曼德布洛特示例所示），还有第三个 操作类型：图形操作。
//
// 在它们用于通用计算之前，GPU 用于图形（因此它们 名称）。为了从中受益，GPU 为开发人员提供了一系列经过优化的专用 称为图形管道的步骤。使用图形管道比 使用计算操作，但它也快得多。

//图形管道的目的是在图像上绘制特定形状。此形状可以是 简单到一个三角形，或者像山脉一样复杂。
//
// 为了启动图形操作（即使用图形管道的操作），您需要 将需要以下元素：
//
// 描述 GPU 应如何运行的图形管道对象，类似于 计算管道对象描述计算操作的方式。
// 包含要绘制的对象形状的一个或多个缓冲区。
// 帧缓冲对象，它是要写入的图像的集合。
// 就像计算管道一样，我们也可以传递描述符集（并推送常量，我们 还没有涵盖）。
// 启动图形操作时，GPU 将通过执行顶点着色器（ 是图形管道对象的一部分），位于要绘制的形状的每个顶点上。这 第一步将允许您在屏幕上定位形状。
//
// 然后，GPU 找出目标图像的哪些像素被形状覆盖，并在每个像素上执行片段着色器（也是图形管道对象的一部分）。这 着色器用于确定给定像素的形状颜色是什么。最后 GPU 会将此颜色与此位置已存在的颜色合并。
//
// 图形管道对象包含顶点着色器、片段着色器以及各种 允许进一步配置图形卡行为的选项。


use std::sync::Arc;
use image::{ImageBuffer, Rgba};
use vulkano::buffer::{BufferContents, BufferUsage};
use vulkano::memory::allocator::MemoryUsage;
use crate::example::buffer::{choose_device, create_auto_command_buffer_builder, create_buffer_allocator, create_device, create_instance, create_iter_buffer, create_memory_allocator, get_queue};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::command_buffer::{CopyImageToBufferInfo, RenderPassBeginInfo, SubpassContents};
use vulkano::format::Format;
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::pipeline::graphics::{input_assembly::InputAssemblyState, vertex_input::Vertex, viewport::{Viewport, ViewportState}};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::Subpass;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use crate::example::pipeline_glsl::{fs, vs};

//顶点数据
#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

pub fn operator_vertex() {
    //三角形顶点数据
    let vertex1 = MyVertex { position: [-0.5, -0.5] };
    let vertex2 = MyVertex { position: [0.0, 0.5] };
    let vertex3 = MyVertex { position: [0.5, -0.25] };

    //创建实例
    let instance = create_instance();
    //选择一台设备
    let physical_device = choose_device(instance);
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

    let memory_allocator_arc = Arc::new(memory_allocator);
    //缓冲区
    let vertex_buffer = create_iter_buffer(
        memory_allocator_arc.clone(),
        BufferUsage::VERTEX_BUFFER,
        MemoryUsage::Upload,
        vec![vertex1, vertex2, vertex3],
    );

    //顶点着色器
    //在绘制操作开始时，GPU 将从此缓冲区中选取每个元素 一个并在它们上调用顶点着色器

    //什么是渲染通道？
    // 术语“渲染通道”描述了两件事：
    //
    // 它指定了我们必须进入的“渲染模式”，然后才能将绘图命令添加到 命令缓冲区。
    //
    // 它还指定一种描述此呈现模式的对象。
    //创建渲染通道
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: Format::R8G8B8A8_UNORM,
                samples: 1
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();
    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![view],
            ..Default::default()
        },
    ).unwrap();

    //创建图形管线
    let vs = vs::load(device.clone()).expect("failed to create shader module");
    let fs = fs::load(device.clone()).expect("failed to create shader module");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [1024.0, 1024.0],
        depth_range: 0.0..1.0,
    };

    let pipeline = GraphicsPipeline::start()
        // Describes the layout of the vertex input and how should it behave
        .vertex_input_state(MyVertex::per_vertex())
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // Indicate the type of the primitives (the default is a list of triangles)
        .input_assembly_state(InputAssemblyState::new())
        // Set the fixed viewport
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        // Same as the vertex input, but this for the fragment input
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        // This graphics pipeline object concerns the first pass of the render pass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that everything is specified, we call `build`.
        .build(device.clone())
        .unwrap();

    //绘图

    //输入一个渲染通道
    let command_buffer_allocator = create_buffer_allocator(device.clone());
    let mut build = create_auto_command_buffer_builder(
        command_buffer_allocator,
        queue.queue_family_index(),
    );

    let buf = create_iter_buffer(
        memory_allocator_arc.clone(),
        BufferUsage::TRANSFER_DST,
        MemoryUsage::Download,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    );

    build
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassContents::Inline,
        ).unwrap()
        .bind_pipeline_graphics(pipeline.clone())
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .draw(3, 1, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image, buf.clone()))
        .unwrap();
    //注意：如果要绘制多个对象，最直接的方法是连续调用多次。draw()

    let command_buffer = build.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image_vertex.png").unwrap();

    println!("Everything vertex succeeded!");
}