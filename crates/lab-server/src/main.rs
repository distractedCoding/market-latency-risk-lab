mod config;
mod wiring;

use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

use runtime::logging::PaperJournalRow;
use runtime::replay::ReplayCsvWriter;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config::Config {
        listen_addr,
        mode,
        replay_output_path,
    } = config::Config::from_env()?;

    println!("{}", startup_mode_banner(mode));
    initialize_replay_output(&replay_output_path)?;
    let listener = TcpListener::bind(listen_addr).await?;

    axum::serve(listener, wiring::build_app()).await?;
    Ok(())
}

fn startup_mode_banner(mode: config::RunMode) -> String {
    format!("lab-server startup mode: {}", mode.as_str())
}

fn initialize_replay_output(path: &str) -> Result<(), std::io::Error> {
    let replay_path = Path::new(path);

    if let Some(parent) = replay_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let replay_file = File::create(replay_path)?;
    let mut replay_writer = ReplayCsvWriter::new(replay_file);
    replay_writer.write_header()?;
    replay_writer.append_paper_journal_rows(&initial_paper_journal_rows())?;
    Ok(())
}

fn initial_paper_journal_rows() -> Vec<PaperJournalRow> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::RunMode;
    use runtime::replay::REPLAY_CSV_HEADER;

    use super::{initialize_replay_output, startup_mode_banner};

    #[test]
    fn initialize_replay_output_creates_parent_dir_and_writes_csv_header() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lab-server-replay-{unique}"));
        let replay_path = root.join("nested").join("replay.csv");

        initialize_replay_output(replay_path.to_str().unwrap())
            .expect("startup should initialize replay output");

        let actual = fs::read_to_string(&replay_path).expect("replay output file should exist");
        assert_eq!(actual, REPLAY_CSV_HEADER);

        fs::remove_dir_all(&root).expect("temp replay directory should be removable");
    }

    #[test]
    fn startup_mode_banner_reports_selected_mode() {
        assert_eq!(
            startup_mode_banner(RunMode::PaperLive),
            "lab-server startup mode: paper-live"
        );
        assert_eq!(
            startup_mode_banner(RunMode::Sim),
            "lab-server startup mode: sim"
        );
    }
}
