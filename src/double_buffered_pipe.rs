use std::io::Write;
use std::thread;

use crate::async_double_buffer::{double_buffer, BufferSide};

pub struct RemoteBufWriter {
    sender: BufferSide<Vec<u8>>,
}

impl RemoteBufWriter {
    pub fn with_capacity<W: Write + Send + 'static>(capacity: usize, writer: W) -> Self {
        let (sender, mut receiver) =
            double_buffer(Vec::with_capacity(capacity), Vec::with_capacity(capacity));

        thread::Builder::new()
            .name("RemoteBufWriter".into())
            .spawn(move || {
                let mut writer = writer;
                while receiver.swap_blocking() {
                    let _ = writer.write_all(&receiver);
                    receiver.clear();
                }
                let _ = writer.flush();
            })
            .unwrap();

        Self { sender }
    }

    fn swap_chunk(&mut self) -> std::io::Result<()> {
        let receiver_alive = self.sender.swap_blocking();
        if !receiver_alive {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Receiver disconnected",
            ));
        }
        Ok(())
    }
}

impl Write for RemoteBufWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let remaining_capacity = self.sender.capacity() - self.sender.len();
        let consumed_len = buf.len().min(remaining_capacity);
        self.sender.extend_from_slice(&buf[..consumed_len]);
        if self.sender.len() == self.sender.capacity() {
            self.swap_chunk()?;
        }
        Ok(consumed_len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.swap_chunk()
    }
}
