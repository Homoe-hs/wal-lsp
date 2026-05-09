use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FstHeader {
    pub timescale_exp: i8,
}

#[derive(Debug, Clone)]
pub struct FstSignal {
    #[allow(dead_code)]
    pub handle: u32,
    pub name: String,
    #[allow(dead_code)]
    pub width: u32,
}

#[derive(Debug, Clone)]
pub struct FstFile {
    pub header: FstHeader,
    pub signals: Vec<FstSignal>,
}

pub struct FstReader<R: Read + Seek> {
    reader: BufReader<R>,
    pub file: FstFile,
}

impl FstReader<File> {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }
}

impl<R: Read + Seek> FstReader<R> {
    pub fn from_reader(reader: R) -> io::Result<Self> {
        let mut r = Self {
            reader: BufReader::new(reader),
            file: FstFile {
                header: FstHeader { timescale_exp: -9 },
                signals: Vec::new(),
            },
        };
        r.read_file()?;
        Ok(r)
    }

    fn read_file(&mut self) -> io::Result<()> {
        loop {
            let block_type = match self.read_u8() {
                Ok(b) => b,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
                Err(e) => return Err(e),
            };

            let block_len = self.read_u64()?;

            match block_type {
                0x00 => self.read_header_block(block_len)?,
                0x01 => self.skip_block(block_len)?,
                0x02 => self.skip_block(block_len)?,
                0x03 => self.read_geom_block(block_len)?,
                0x04 => self.read_hier_block(block_len)?,
                0x06 => self.read_hier_lz4_block(block_len)?,
                0x07 => self.read_hier_lz4_block(block_len)?,
                0xFE => self.read_zwrapper_block(block_len)?,
                _ => self.skip_block(block_len)?,
            }
        }
    }

    #[inline]
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_u64(&mut self) -> io::Result<u64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    #[inline]
    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        if len > 64 * 1024 * 1024 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Block too large (>64MB)"));
        }
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn skip_block(&mut self, len: u64) -> io::Result<()> {
        let signed_len = if len > i64::MAX as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Block length too large"));
        } else {
            len as i64
        };
        self.reader.seek(SeekFrom::Current(signed_len))?;
        Ok(())
    }

    fn read_header_block(&mut self, len: u64) -> io::Result<()> {
        let start_pos = self.reader.stream_position()?;

        self.file.header.timescale_exp = self.read_i8()?;

        let end_pos = start_pos + len as u64;
        if self.reader.stream_position()? < end_pos {
            self.reader.seek(SeekFrom::Start(end_pos))?;
        }
        Ok(())
    }

    #[inline]
    fn read_i8(&mut self) -> io::Result<i8> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    fn read_geom_block(&mut self, len: u64) -> io::Result<()> {
        let start_pos = self.reader.stream_position()?;
        let _section_length = self.read_u64()?;
        let _uncompressed_length = self.read_u64()?;
        let _max_handle = self.read_u64()?;

        let end_pos = start_pos + len as u64;
        while self.reader.stream_position()? < end_pos {
            let handle = self.read_u32()?;
            let varint_bytes = self.read_varint_bytes()?;
            let (name_len, _) = decode_varint(&varint_bytes)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid varint"))?;
            let name_bytes = self.read_bytes(name_len as usize)?;
            let name = String::from_utf8_lossy(&name_bytes).to_string();
            let _var_type = self.read_u8()?;
            let _direction = self.read_u8()?;
            let width = self.read_u32()?;

            self.file.signals.push(FstSignal {
                handle,
                name,
                width,
            });
        }

        if self.reader.stream_position()? != end_pos {
            self.reader.seek(SeekFrom::Start(end_pos))?;
        }

        Ok(())
    }

    fn read_hier_block(&mut self, len: u64) -> io::Result<()> {
        self.skip_block(len)
    }

    fn read_hier_lz4_block(&mut self, len: u64) -> io::Result<()> {
        let compressed = self.read_bytes(len as usize)?;
        let _decompressed = lz4_flex::block::decompress_size_prepended(&compressed)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("LZ4 decompression failed: {}", e)))?;
        Ok(())
    }

    fn read_zwrapper_block(&mut self, len: u64) -> io::Result<()> {
        let compressed = self.read_bytes(len as usize)?;

        let decompressed = {
            use flate2::read::ZlibDecoder;
            use std::io::Read;
            let mut decoder = ZlibDecoder::new(&compressed[..]);
            let mut output = Vec::new();
            decoder.read_to_end(&mut output)?;
            output
        };

        let mut cursor = std::io::Cursor::new(decompressed);
        let inner_reader = FstReader::from_reader(&mut cursor)?;

        self.file.header = inner_reader.file.header;
        self.file.signals.extend(inner_reader.file.signals);

        Ok(())
    }

    fn read_varint_bytes(&mut self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(10);
        loop {
            let b = self.read_u8()?;
            buf.push(b);
            if b & 0x80 == 0 {
                break;
            }
        }
        Ok(buf)
    }
}

#[inline]
pub fn decode_varint(buf: &[u8]) -> Option<(u64, usize)> {
    if buf.is_empty() {
        return None;
    }

    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = 0;

    loop {
        if pos >= buf.len() {
            return None;
        }
        let b = buf[pos];
        result |= ((b & 0x7F) as u64) << shift;
        pos += 1;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }

    Some((result, pos))
}

impl FstFile {
    pub fn signal_names(&self) -> Vec<String> {
        self.signals.iter().map(|s| s.name.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_decode() {
        assert_eq!(decode_varint(&[0x00]), Some((0, 1)));
        assert_eq!(decode_varint(&[0x7F]), Some((127, 1)));
        assert_eq!(decode_varint(&[0x80, 0x01]), Some((128, 2)));
        assert_eq!(decode_varint(&[0xFF, 0x7F]), Some((16383, 2)));
        assert_eq!(decode_varint(&[0xFF, 0xFF, 0x7F]), Some((2097151, 3)));
    }

    #[test]
    fn test_decode_varint_edge_cases() {
        // min values
        assert_eq!(decode_varint(&[0x00]), Some((0, 1)));
        assert_eq!(decode_varint(&[0x7F]), Some((127, 1)));
        assert_eq!(decode_varint(&[0x80, 0x01]), Some((128, 2)));
        // max 2-byte
        assert_eq!(decode_varint(&[0xFF, 0x7F]), Some((16383, 2)));
        // max 3-byte
        assert_eq!(decode_varint(&[0xFF, 0xFF, 0x7F]), Some((2097151, 3)));
        // 4-byte value
        assert_eq!(decode_varint(&[0x80, 0x80, 0x80, 0x01]), Some((2097152, 4)));
        // incomplete
        assert_eq!(decode_varint(&[0x80]), None);
        // empty
        assert_eq!(decode_varint(&[]), None);
    }

    #[test]
    fn test_decode_varint_many_values() {
        // Sequential varint: 0x01(=1) 0x7F(=127) 0x80 0x01(=128) 0xFF 0x7F(=16383)
        let data = [0x01, 0x7F, 0x80, 0x01, 0xFF, 0x7F];
        let (val1, n1) = decode_varint(&data[0..]).unwrap();
        let (val2, n2) = decode_varint(&data[n1..]).unwrap();
        let (val3, n3) = decode_varint(&data[n1+n2..]).unwrap();
        let (val4, n4) = decode_varint(&data[n1+n2+n3..]).unwrap();
        assert_eq!(val1, 1);
        assert_eq!(val2, 127);
        assert_eq!(val3, 128);
        assert_eq!(val4, 16383);
        assert_eq!(n1 + n2 + n3 + n4, 6);
    }
}
