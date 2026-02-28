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
        tick: u64,
        run_log_writer: &mut dyn RunLogWriter,
    ) -> io::Result<()> {
        self.write_header()?;
        self.writer.flush()?;
        run_log_writer.write(RunLogEvent::new(
            tick,
            RunLogEventKind::ReplayArtifactWritten,
            None,
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        io,
        rc::Rc,
    };

    use crate::logging::{InMemoryRunLogWriter, RunLogEventKind, RunLogWriter};

    use super::{REPLAY_CSV_HEADER, ReplayCsvWriter};

    struct TrackingWriter {
        bytes: Vec<u8>,
        flush_called: Rc<Cell<bool>>,
        flush_fails: bool,
    }

    impl TrackingWriter {
        fn new(flush_called: Rc<Cell<bool>>, flush_fails: bool) -> Self {
            Self {
                bytes: Vec::new(),
                flush_called,
                flush_fails,
            }
        }
    }

    impl io::Write for TrackingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flush_called.set(true);
            if self.flush_fails {
                return Err(io::Error::other("flush failed"));
            }
            Ok(())
        }
    }

    struct FlushAssertingLogWriter {
        flush_called: Rc<Cell<bool>>,
    }

    impl RunLogWriter for FlushAssertingLogWriter {
        fn write(&mut self, _event: crate::logging::RunLogEvent) {
            assert!(self.flush_called.get(), "expected writer flush before logging");
        }
    }

    #[test]
    fn write_header_and_log_flushes_before_emitting_log() {
        let flush_called = Rc::new(Cell::new(false));
        let writer = TrackingWriter::new(Rc::clone(&flush_called), false);
        let mut replay_writer = ReplayCsvWriter::new(writer);
        let mut log_writer = FlushAssertingLogWriter { flush_called };

        replay_writer
            .write_header_and_log(7, &mut log_writer)
            .expect("header write should flush and log");
    }

    #[test]
    fn write_header_and_log_propagates_flush_errors() {
        let flush_called = Rc::new(Cell::new(false));
        let writer = TrackingWriter::new(Rc::clone(&flush_called), true);
        let mut replay_writer = ReplayCsvWriter::new(writer);
        let mut log_writer = InMemoryRunLogWriter::new();

        let err = replay_writer
            .write_header_and_log(3, &mut log_writer)
            .expect_err("flush failure should be returned");

        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(log_writer.events().len(), 0);
    }

    #[test]
    fn write_header_and_log_uses_tick_from_caller() {
        let mut output = Vec::new();
        let mut replay_writer = ReplayCsvWriter::new(&mut output);
        let mut log_writer = InMemoryRunLogWriter::new();

        replay_writer
            .write_header_and_log(42, &mut log_writer)
            .expect("header and log write should succeed");

        assert_eq!(String::from_utf8(output).unwrap(), REPLAY_CSV_HEADER);
        assert_eq!(log_writer.events().len(), 1);
        assert_eq!(log_writer.events()[0].tick, 42);
        assert_eq!(log_writer.events()[0].kind, RunLogEventKind::ReplayArtifactWritten);
    }
}
