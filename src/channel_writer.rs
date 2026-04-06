use std::io;
use std::ops::{Deref, DerefMut};

use bytes::Bytes;
use tokio_stream::wrappers::ReceiverStream;

pub fn writer_with_stream(
    buffers: Vec<Vec<u8>>,
) -> (
    BlockingChannelWriter,
    ReceiverStream<Result<Bytes, io::Error>>,
) {
    let (pool_tx, pool_rx) = std::sync::mpsc::sync_channel(buffers.len());
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(buffers.len());
    for buffer in buffers {
        pool_tx.send(buffer).unwrap();
    }
    let writer = BlockingChannelWriter {
        pool_rx,
        pool_tx,
        output_tx,
    };
    let stream = ReceiverStream::new(output_rx);
    (writer, stream)
}

/// A writer that sends a message for every write. Wrap it in a BufWriter.
pub struct BlockingChannelWriter {
    pool_rx: std::sync::mpsc::Receiver<Vec<u8>>,
    pool_tx: std::sync::mpsc::SyncSender<Vec<u8>>,
    output_tx: tokio::sync::mpsc::Sender<Result<Bytes, io::Error>>,
}

pub struct Buffer {
    pool_tx: std::sync::mpsc::SyncSender<Vec<u8>>,
    buffer: Vec<u8>,
}

impl io::Write for BlockingChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !buf.is_empty() {
            let pool_tx = self.pool_tx.clone();
            let mut buffer = self.pool_rx.recv().unwrap();
            buffer.clear();
            buffer.extend_from_slice(buf);
            let bytes = Bytes::from_owner(Buffer { pool_tx, buffer });
            self.output_tx
                .blocking_send(Ok(bytes))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let _ = self.pool_tx.send(std::mem::take(&mut self.buffer));
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}
