use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("[sample-service] Starting sample-service v0.1.0...");
    println!("[sample-service] Listening for requests on localhost:8080");

    for i in 1..=5 {
        println!("[sample-service] Heartbeat #{} - system healthy", i);
        sleep(Duration::from_millis(500));
    }

    println!("[sample-service] Completed execution cycle.");
}
