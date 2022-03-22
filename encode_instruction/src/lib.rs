pub trait EncodeInstruction: Sized + Copy {
    fn encode(self, buf: &mut Vec<u8>);
    fn decode(buf: &[u8]) -> Option<(Self, usize)>;
}

impl EncodeInstruction for u8 {
    fn encode(self, buf: &mut Vec<u8>) {
        buf.push(self)
    }

    fn decode(buf: &[u8]) -> Option<(Self, usize)> {
        Some((buf[0], 1))
    }
}

impl EncodeInstruction for usize {
    fn encode(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes())
    }

    fn decode(buf: &[u8]) -> Option<(Self, usize)> {
        let size = std::mem::size_of::<usize>();

        Some((usize::from_be_bytes(buf[0..size].try_into().ok()?), size))
    }
}
