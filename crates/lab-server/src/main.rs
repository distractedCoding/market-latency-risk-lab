mod config;
mod wiring;

use std::error::Error;
use std::env;
use std::fs::{self, File};
use std::path::Path;

use runtime::logging::{PaperJournalRow, PaperJournalRowKind};
use runtime::replay::ReplayCsvWriter;
use tokio::net::TcpListener;

const BOOTSTRAP_ROWS_ENV: &str = "LAB_SERVER_INITIAL_PAPER_JOURNAL_ROWS";

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
    let Ok(value) = env::var(BOOTSTRAP_ROWS_ENV) else {
        return Vec::new();
    };

    value
        .split(';')
        .filter_map(parse_bootstrap_paper_journal_row)
        .collect()
}

fn parse_bootstrap_paper_journal_row(value: &str) -> Option<PaperJournalRow> {
    let mut parts = value.splitn(3, '|');
    let tick = parts.next()?.trim().parse::<u64>().ok()?;
    let kind = match parts.next()?.trim() {
        "paper_fill" => PaperJournalRowKind::PaperFill,
        _ => return None,
    };
    let action_detail = parts.next()?.trim();
    if action_detail.is_empty() {
        return None;
    }

    Some(PaperJournalRow {
        tick,
        kind,
        action_detail: action_detail.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    use runtime::logging::PaperJournalRowKind;
    use crate::config::RunMode;
    use runtime::replay::REPLAY_CSV_HEADER;

    use super::{initial_paper_journal_rows, initialize_replay_output, startup_mode_banner};

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    const ENV_BOOTSTRAP_ROWS: &str = "LAB_SERVER_INITIAL_PAPER_JOURNAL_ROWS";

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = env::var_os(key);
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn initialize_replay_output_creates_parent_dir_and_writes_csv_header() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _bootstrap_guard = EnvVarGuard::unset(ENV_BOOTSTRAP_ROWS);
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

    #[test]
    fn initial_paper_journal_rows_is_empty_without_bootstrap_env() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _guard = EnvVarGuard::unset(ENV_BOOTSTRAP_ROWS);

        let rows = initial_paper_journal_rows();

        assert!(rows.is_empty());
    }

    #[test]
    fn initial_paper_journal_rows_reads_bootstrap_rows_from_env() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _guard = EnvVarGuard::set(
            ENV_BOOTSTRAP_ROWS,
            "17|paper_fill|buy:market-1@0.62x5;18|paper_fill|sell:market-2@0.41x2",
        );

        let rows = initial_paper_journal_rows();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tick, 17);
        assert_eq!(rows[0].kind, PaperJournalRowKind::PaperFill);
        assert_eq!(rows[0].action_detail, "buy:market-1@0.62x5");
        assert_eq!(rows[1].tick, 18);
        assert_eq!(rows[1].kind, PaperJournalRowKind::PaperFill);
        assert_eq!(rows[1].action_detail, "sell:market-2@0.41x2");
    }

    #[test]
    fn initialize_replay_output_appends_bootstrap_rows_when_provided() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _guard = EnvVarGuard::set(ENV_BOOTSTRAP_ROWS, "17|paper_fill|buy:market-1@0.62x5");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lab-server-replay-bootstrap-{unique}"));
        let replay_path = root.join("nested").join("replay.csv");

        initialize_replay_output(replay_path.to_str().unwrap())
            .expect("startup should initialize replay output");

        let actual = fs::read_to_string(&replay_path).expect("replay output file should exist");
        assert_eq!(
            actual,
            format!(
                "{REPLAY_CSV_HEADER}17,,,,paper_fill:buy:market-1@0.62x5,,,,\n"
            )
        );

        fs::remove_dir_all(&root).expect("temp replay directory should be removable");
    }
}
