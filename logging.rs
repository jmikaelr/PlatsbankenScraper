use flexi_logger::{Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming, WriteMode};
pub use log::{error, info};

pub async fn setup_log() -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str("info")
        .map_err(|e| format!("Logger initialization failed: {}", e))?
        .log_to_file(FileSpec::default().directory("logs").basename("app"))
        .duplicate_to_stderr(Duplicate::All)
        .rotate(Criterion::Age(Age::Day), Naming::Timestamps, Cleanup::Never)
        .write_mode(WriteMode::Direct)
        .start()
        .map_err(|e| format!("Logger start failed: {}", e))?;
    info!("Logger successfully initialized!");
    Ok(())
}
