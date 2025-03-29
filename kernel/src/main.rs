#![feature(sync_unsafe_cell)]
#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

mod screen;
mod allocator;
mod frame_allocator;
mod interrupts;
mod gdt;
mod pong;

use alloc::boxed::Box;
use core::fmt::Write;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping::Dynamic;
use bootloader_api::info::MemoryRegionKind;
use kernel::{HandlerTable, serial};
use pc_keyboard::DecodedKey;
use x86_64::VirtAddr;
use crate::frame_allocator::BootInfoFrameAllocator;
use crate::pong::PongGame;
use crate::screen::Writer;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Dynamic); // obtain physical memory offset
    config.kernel_stack_size = 256 * 1024; // 256 KiB kernel stack size
    config
};
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

// Global game state
static mut GAME: Option<PongGame> = None;

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    writeln!(serial(), "Entered kernel with boot info: {boot_info:?}").unwrap();
    writeln!(serial(), "Frame Buffer: {:p}", boot_info.framebuffer.as_ref().unwrap().buffer()).unwrap();

    let frame_info = boot_info.framebuffer.as_ref().unwrap().info();
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    screen::init(framebuffer);
    
    writeln!(serial(), "Screen initialized with dimensions: {}x{}", 
             frame_info.width, frame_info.height).unwrap();

    for r in boot_info.memory_regions.iter() {
        writeln!(serial(), "{:?} {:?} {:?} {}", r, r.start as *mut u8, r.end as *mut usize, r.end-r.start).unwrap();
    }

    let usable_region = boot_info.memory_regions.iter().filter(|x|x.kind == MemoryRegionKind::Usable).last().unwrap();
    writeln!(serial(), "{usable_region:?}").unwrap();

    let physical_offset = boot_info.physical_memory_offset.take().expect("Failed to find physical memory offset");
    let ptr = (physical_offset + usable_region.start) as *mut u8;
    writeln!(serial(), "Physical memory offset: {:X}; usable range: {:p}", physical_offset, ptr).unwrap();

    // Initialize heap allocator
    allocator::init_heap((physical_offset + usable_region.start) as usize);

    let rsdp = boot_info.rsdp_addr.take();
    let mut mapper = frame_allocator::init(VirtAddr::new(physical_offset));
    let mut frame_allocator = BootInfoFrameAllocator::new(&boot_info.memory_regions);
    
    gdt::init();

    // Test heap allocation
    let x = Box::new(42);
    let y = Box::new(24);
    writeln!(Writer, "Heap allocation works: {} + {} = {}", *x, *y, *x + *y).unwrap();
    
    writeln!(serial(), "Starting kernel and initializing Pong game...").unwrap();
    
    // Initialize Pong game
    unsafe {
        GAME = Some(PongGame::new(frame_info.width as usize, frame_info.height as usize));
    }

    let lapic_ptr = interrupts::init_apic(rsdp.expect("Failed to get RSDP address") as usize, physical_offset, &mut mapper, &mut frame_allocator);
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(start)
        .start(lapic_ptr)
}

fn start() {
    writeln!(Writer, "Welcome to Pong OS!").unwrap();
    writeln!(Writer, "Use Up/Down arrows to move your paddle").unwrap();
    writeln!(Writer, "First to 5 points wins!").unwrap();
    
    // Initial render of the game using raw pointer
    unsafe {
        let game_ptr = &raw const GAME;
        if let Some(game) = &*game_ptr {
            game.render();
        }
    }
}

fn tick() {
    unsafe {
        let game_ptr = &raw mut GAME;
        if let Some(game) = &mut *game_ptr {
            game.update();
            game.render();
        }
    }
}

fn key(key: DecodedKey) {
    unsafe {
        let game_ptr = &raw mut GAME;
        if let Some(game) = &mut *game_ptr {
            game.handle_key(key);
        }
    }
}