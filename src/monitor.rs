use crate::cli::AiEngine;
use colored::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn monitor_progress(task: String, engine: AiEngine) {
    let start = Instant::now();
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let mut spin_idx = 0;

    let task_display: String = task.chars().take(40).collect();

    loop {
        let elapsed = start.elapsed();
        let mins = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;

        let spinner = spinner_chars[spin_idx];
        let step = "Processing";

        print!(
            "\r  {} {} │ {} {}",
            spinner.to_string().cyan(),
            format!("{:16}", step).bright_cyan(),
            task_display,
            format!("[{:02}:{:02}]", mins, secs).bright_black()
        );
        use std::io::Write;
        std::io::stdout().flush().ok();

        spin_idx = (spin_idx + 1) % spinner_chars.len();
        sleep(Duration::from_millis(120)).await;
    }
}
