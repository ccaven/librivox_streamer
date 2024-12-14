pub struct ReadableReceiver {
    receiver: flume::Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    length: usize
}

impl ReadableReceiver {
    pub fn new(
        receiver: flume::Receiver<Vec<u8>>, 
        buffer_size: usize
    ) -> Self {
        let buffer = vec![0u8; buffer_size];
        let length = 0;

        ReadableReceiver {
            receiver,
            buffer,
            length
        }
    }
}

impl std::io::Read for ReadableReceiver {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while self.length < buf.len() {
            if let Ok(chunk) = self.receiver.recv() {
                let space = self.buffer.len() - self.length;

                if space > chunk.len() {
                    for i in 0..chunk.len() {
                        self.buffer[self.length + i] = chunk[i];
                    }

                } else {
                    for i in self.length..self.buffer.len() {
                        self.buffer[i] = chunk[i - self.length];
                    }

                    for i in self.buffer.len()..(self.length+chunk.len()) {
                        self.buffer.push(chunk[i - self.length]);
                    }
                }

                self.length += chunk.len();
            } else {
                break;
            }
        }

        let available = if self.length > buf.len() { buf.len() } else { self.length };
        
        for i in 0..available {
            buf[i] = self.buffer[i];
        }

        self.buffer.copy_within(available..self.length, 0);

        self.length -= available;

        Ok(available)
    }
}
