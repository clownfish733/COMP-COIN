use COMP_COIN::start_server;

use anyhow::Result;

use std::io::Write;

use log::LevelFilter;

fn main() -> Result<()>{

    //Configure logging
    env_logger::builder()
        .format(|buf, record| {
            // Color by log level
            let level_color = match record.level() {
                log::Level::Error => "\x1b[31m", // Red
                log::Level::Warn => "\x1b[33m",  // Yellow
                log::Level::Info => "\x1b[32m",  // Green
                log::Level::Debug => "\x1b[36m", // Cyan
                log::Level::Trace => "\x1b[90m", // Gray
            };
            
            // Color by module/target
            let target = record.target();
            let module_color = match target {
                t if t.contains("network") => "\x1b[35m", // Magenta for network
                t if t.contains("mine") => "\x1b[33m",    // Yellow for mining
                t if t.contains("ui") => "\x1b[36m",      // Cyan for UI
                t if t.contains("block") => "\x1b[32m",   // Green for block
                t if t.contains("node") => "\x1b[34m",    // Blue for node
                t if t.contains("utils") => "\x1b[90m",   // Bright black/gray for utils
                _ => "\x1b[37m",                          // White for others
            };
            
            writeln!(
                buf,
                "{}{:<5}\x1b[0m {}[{}]\x1b[0m {}",
                level_color,
                record.level(),
                module_color,
                target,
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .init();

    //runtime setup
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(64 * 1024 * 1024)
        .enable_all()
        .build()?;

    runtime.block_on(start_server())?;
    Ok(())
}