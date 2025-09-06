use std::io;

use bytes::Bytes;
use tokio::sync::mpsc;

/// A writer that sends a message for every write. Wrap it in a BufWriter.
pub struct BlockingChannelWriter(mpsc::Sender<Result<Bytes, io::Error>>);

impl BlockingChannelWriter {
    pub fn new(sender: mpsc::Sender<Result<Bytes, io::Error>>) -> Self {
        Self(sender)
    }
}

impl io::Write for BlockingChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !buf.is_empty() {
            let bytes = Bytes::from(buf.to_owned());
            self.0
                .blocking_send(Ok(bytes))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
