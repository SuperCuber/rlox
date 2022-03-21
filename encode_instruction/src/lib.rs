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
