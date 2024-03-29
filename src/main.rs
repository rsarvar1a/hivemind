use clap::Parser;
use flexi_logger::{AdaptiveFormat, Logger, WriteMode};
use hivemind::prelude::*;

fn main() -> Result<()>
{
    // Read the dotenv file.
    dotenvy::dotenv().ok();

    // Parse the cli options.
    let options = UhpOptions::parse();

    // Choose the logging type based on build type.
    let log_format = if cfg!(debug_assertions)
    {
        AdaptiveFormat::WithThread
    }
    else
    {
        AdaptiveFormat::Default
    };

    // Set the logger to write-and-flush so that it doesn't compete with worker threads.
    let _logger = Logger::try_with_env_or_str(options.log_level.clone())?
        .write_mode(WriteMode::BufferAndFlush)
        .log_to_stderr()
        .adaptive_format_for_stderr(log_format)
        .set_palette("b196;208;195;111;67".to_owned())
        .start()?;

    // Display the server package information.
    print_header();

    // Run the main UHP loop.
    if let Err(e) = Server::<evaluators::Strongest>::new(options).run()
    {
        log::error!("fatal error: {}", e);
    }

    Ok(())
}

fn print_header()
{
    println!("");
    log::info!("🐝 starting {} server v{} 🐝", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    log::debug!("here be bugs 🐜 🪲  🦗 🐞 🦟 🦠 🕷️");
    log::trace!("... you poor soul.");
}
