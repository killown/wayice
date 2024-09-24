use wayice::platform::udev;
use wayice::platform::winit;
use wayice::platform::x11;

static POSSIBLE_BACKENDS: &[&str] = &[
    #[cfg(feature = "winit")]
    "--winit : Run wayice as a X11 or Wayland client using winit.",
    #[cfg(feature = "udev")]
    "--tty-udev : Run wayice as a tty udev client (requires root if without logind).",
    #[cfg(feature = "x11")]
    "--x11 : Run wayice as an X11 client.",
];

#[cfg(feature = "profile-with-tracy-mem")]
#[global_allocator]
static GLOBAL: profiling::tracy_client::ProfiledAllocator<std::alloc::System> =
    profiling::tracy_client::ProfiledAllocator::new(std::alloc::System, 10);

use drm_fourcc::{DrmFourcc, DrmModifier};
use smithay::backend::{
    allocator::{
        dmabuf::AsDmabuf,
        vulkan::{ImageUsageFlags, VulkanAllocator},
        Allocator, Buffer,
    },
    vulkan::{version::Version, Instance, PhysicalDevice},
};

fn vulkan_init() {
    // Initialize tracing for logging purposes.
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    // Print available Vulkan instance extensions.
    println!(
        "Available instance extensions: {:?}",
        Instance::enumerate_extensions().unwrap().collect::<Vec<_>>()
    );
    println!();

    // Create a new Vulkan instance, targeting Vulkan API version 1.3.
    let instance = Instance::new(Version::VERSION_1_3, None).unwrap();

    // Enumerate all physical devices (GPUs) and print their details.
    for (idx, phy) in PhysicalDevice::enumerate(&instance).unwrap().enumerate() {
        println!(
            "Device #{}: {} v{}, {:?}",
            idx,
            phy.name(),
            phy.api_version(),
            phy.driver()
        );
    }

    // Select the first available physical device (GPU).
    let physical_device = PhysicalDevice::enumerate(&instance)
        .unwrap()
        .next()
        .expect("No physical devices available");

    // Initialize VulkanAllocator to create buffers suitable as render targets.
    let mut allocator = VulkanAllocator::new(&physical_device, ImageUsageFlags::COLOR_ATTACHMENT).unwrap();

    // Create a buffer of 100x200 dimensions with ARGB8888 pixel format and Linear DRM modifier.
    let image = allocator
        .create_buffer(100, 200, DrmFourcc::Argb8888, &[DrmModifier::Linear])
        .expect("Failed to create buffer");

    // Check if the buffer's dimensions are correct.
    assert_eq!(image.width(), 100);
    assert_eq!(image.height(), 200);

    // Export the buffer as a DMA-BUF for inter-process sharing.
    let image_dmabuf = image.export().expect("Failed to export DMA-BUF");

    // Drop the image to release resources.
    drop(image);

    // Create another buffer of 200x200 dimensions.
    let _image2 = allocator
        .create_buffer(200, 200, DrmFourcc::Argb8888, &[DrmModifier::Linear])
        .expect("Failed to create second buffer");

    // Clean up resources by dropping the allocator and DMA-BUF handle.
    drop(allocator);
    drop(image_dmabuf);
}

fn main() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().compact().init();
    }

    #[cfg(feature = "profile-with-tracy")]
    profiling::tracy_client::Client::start();

    profiling::register_thread!("Main Thread");

    #[cfg(feature = "profile-with-puffin")]
    let _server = puffin_http::Server::new(&format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT)).unwrap();
    #[cfg(feature = "profile-with-puffin")]
    profiling::puffin::set_scopes_on(true);

    let arg = ::std::env::args().nth(1);
    match arg.as_ref().map(|s| &s[..]) {
        #[cfg(feature = "winit")]
        Some("--winit") => {
            tracing::info!("Starting wayice with winit backend");
            winit::run_winit();
        }

        Some("--vulkan") => {
            tracing::info!("Starting wayice with winit backend");
            vulkan_init();
        }
        #[cfg(feature = "udev")]
        Some("--tty-udev") => {
            tracing::info!("Starting wayice on a tty using udev");
            udev::run_udev();
        }
        #[cfg(feature = "x11")]
        Some("--x11") => {
            tracing::info!("Starting wayice with x11 backend");
            x11::run_x11();
        }
        Some(other) => {
            tracing::error!("Unknown backend: {}", other);
        }
        None => {
            #[allow(clippy::disallowed_macros)]
            {
                println!("USAGE: wayice --backend");
                println!();
                println!("Possible backends are:");
                for b in POSSIBLE_BACKENDS {
                    println!("\t{}", b);
                }
            }
        }
    }
}
