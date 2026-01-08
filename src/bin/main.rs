use COMP_COIN::node::{full_node_main, bootstrap_node_main};
use anyhow::{Result, anyhow};
use std::{
    env,
    io::Write,
};
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
            let module_color = if target.contains("network") {
                "\x1b[35m" // Magenta for network
            } else if target.contains("mine") {
                "\x1b[33m" // Yellow for mining
            } else if target.contains("ui") {
                "\x1b[36m" // Cyan for UI
            } else {
                "\x1b[37m" // White for others
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

    //paramater handling
    let load = match env::args().nth(2).as_deref(){
        Some("load") => true, 
        Some("new") => false,
        Some(arg) => return Err(anyhow!("invalid arguement: '{}' expected 'new' or 'load'", arg)),
        None => return Err(anyhow!("Missing arguement: expected: 'load' or 'new"))
    };
    

    //runtime setup
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(64 * 1024 * 1024)
        .enable_all()
        .build()?;

    match env::args().nth(1).as_deref(){
        Some("Bootstrap") => runtime.block_on(bootstrap_node_main(load)),
        Some("Full-Node") => runtime.block_on(full_node_main(load)),
        Some(arg) => return Err(anyhow!("Invalid arguement '{}' expected 'Bootstrap' or 'Full-Node'", arg)),
        None => return Err(anyhow!("Missing argument: expected: 'Bootstrap' or 'Full-Node"))
    }
}