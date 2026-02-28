use std::io::{self, Write};

use crate::logging::{RunLogEvent, RunLogEventKind, RunLogWriter};

pub const REPLAY_CSV_HEADER: &str =
    "t,external_px,market_px,divergence,action,equity,realized_pnl,position,halted\n";

pub struct ReplayCsvWriter<W: Write> {
    writer: W,
}

impl<W: Write> ReplayCsvWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_header(&mut self) -> io::Result<()> {
        self.writer.write_all(REPLAY_CSV_HEADER.as_bytes())
    }

    pub fn write_header_and_log(
        &mut self,
        run_log_writer: &mut dyn RunLogWriter,
    ) -> io::Result<()> {
        self.write_header()?;
        run_log_writer.write(RunLogEvent::new(
            0,
            RunLogEventKind::ReplayArtifactWritten,
            None,
        ));
        Ok(())
    }
}
