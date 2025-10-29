/// Custom panic handler for better debugging
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    // Print panic information
    esp_println::println!("\n=== PANIC OCCURRED ===");

    // Print panic location if available
    if let Some(location) = info.location() {
        esp_println::println!(
            "Panic occurred at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        esp_println::println!("Panic occurred at unknown location");
    }

    // Print panic message if available
    let message = info.message();
    esp_println::println!("Panic message: {}", message);

    // Print memory information
    esp_println::println!("\n=== MEMORY INFO ===");
    esp_println::println!("Stack pointer: unavailable (assembly removed)");

    // Print some general debug info
    esp_println::println!("\n=== DEBUG INFO ===");
    esp_println::println!("Target: unknown"); // The TARGET env variable is not set during runtime
    esp_println::println!(
        "Profile: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    // Force a flush to ensure all output is printed
    esp_println::println!("\n=== ENTERING PANIC LOOP ===");

    // Custom panic handler loop - no automatic reset
    // The system will remain in this loop until manually reset
    loop {
        // Small delay to prevent overwhelming the output
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
}
