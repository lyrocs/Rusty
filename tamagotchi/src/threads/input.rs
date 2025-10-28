// Input thread implementation
//
// This thread runs on Core 1 and continuously polls touch and button inputs

use crate::drivers::touch::SharedTouch;
use crate::types::InputEvent;
use crossbeam_channel::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Spawn the input thread
pub fn spawn_input_thread(
    touch: SharedTouch,
    tx: Sender<InputEvent>,
    running: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("input".to_string())
        .stack_size(4096)
        .spawn(move || {
            log::info!("Input thread started");

            let mut last_touch = None;

            while running.load(Ordering::Relaxed) {
                // Poll touch input at 120Hz for responsive input
                if let Some(mut touch_driver) = touch.try_lock() {
                    if let Some(pos) = touch_driver.read_touch() {
                        if last_touch != Some(pos) {
                            tx.send(InputEvent::Touch(pos.0, pos.1)).ok();
                            last_touch = Some(pos);
                        }
                    } else if last_touch.is_some() {
                        tx.send(InputEvent::TouchRelease).ok();
                        last_touch = None;
                    }
                }

                // Sleep longer to avoid watchdog issues on ESP32
                // Reduced polling rate but ensures IDLE task can run
                thread::sleep(Duration::from_millis(50));
            }

            log::info!("Input thread stopped");
        })
        .expect("Failed to spawn input thread")
}
