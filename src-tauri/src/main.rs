// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        for (key, val) in [
            ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
            ("WEBKIT_DISABLE_COMPOSITING_MODE", "1"),
        ] {
            if std::env::var_os(key).is_none() {
                unsafe { std::env::set_var(key, val) };
            }
        }
        if std::env::var_os("LD_PRELOAD").is_none() {
            for path in [
                "/usr/lib64/libwayland-client.so.0",                // Fedora
                "/usr/lib/x86_64-linux-gnu/libwayland-client.so.0", // Debian
                "/usr/lib/libwayland-client.so.0",                  // Arch
            ] {
                if std::path::Path::new(path).exists() {
                    unsafe { std::env::set_var("LD_PRELOAD", path) };
                    break;
                }
            }
        }
    }

    null_launcher_lib::run()
}
