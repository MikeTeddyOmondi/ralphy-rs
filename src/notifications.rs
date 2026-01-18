use notify_rust::Notification;

pub fn notify_done(message: &str) {
    #[cfg(target_os = "macos")]
    {
        // macOS sound
        std::process::Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        // Linux sound
        std::process::Command::new("paplay")
            .arg("/usr/share/sounds/freedesktop/stereo/complete.oga")
            .spawn()
            .ok();
    }

    // Send notification
    Notification::new()
        .summary("Ralphy")
        .body(message)
        .show()
        .ok();
}

pub fn notify_error(message: &str) {
    Notification::new()
        .summary("Ralphy - Error")
        .body(message)
        .show()
        .ok();
}
