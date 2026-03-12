use std::io;

pub struct EndianReader<'a> {
    data: &'a [u8],
    pos: usize,
    big_endian: bool,
}

impl<'a> EndianReader<'a> {
    pub fn new(data: &'a [u8], big_endian: bool) -> Self {
        Self {
            data,
            pos: 0,
            big_endian,
        }
    }

    pub fn set_big_endian(&mut self, big_endian: bool) {
        self.big_endian = big_endian;
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    #[allow(dead_code)]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_i8(&mut self) -> Result<i8, io::Error> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u8(&mut self) -> Result<u8, io::Error> {
        self.ensure(1)?;
        let val = self.data[self.pos];
        self.pos += 1;
        Ok(val)
    }

    pub fn read_i16(&mut self) -> Result<i16, io::Error> {
        self.ensure(2)?;
        let bytes: [u8; 2] = self.data[self.pos..self.pos + 2].try_into().unwrap();
        self.pos += 2;
        Ok(if self.big_endian {
            i16::from_be_bytes(bytes)
        } else {
            i16::from_le_bytes(bytes)
        })
    }

    pub fn read_u16(&mut self) -> Result<u16, io::Error> {
        self.ensure(2)?;
        let bytes: [u8; 2] = self.data[self.pos..self.pos + 2].try_into().unwrap();
        self.pos += 2;
        Ok(if self.big_endian {
            u16::from_be_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        })
    }

    pub fn read_i32(&mut self) -> Result<i32, io::Error> {
        self.ensure(4)?;
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        Ok(if self.big_endian {
            i32::from_be_bytes(bytes)
        } else {
            i32::from_le_bytes(bytes)
        })
    }

    pub fn read_u32(&mut self) -> Result<u32, io::Error> {
        self.ensure(4)?;
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        Ok(if self.big_endian {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        })
    }

    pub fn read_f32(&mut self) -> Result<f32, io::Error> {
        self.ensure(4)?;
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        Ok(if self.big_endian {
            f32::from_be_bytes(bytes)
        } else {
            f32::from_le_bytes(bytes)
        })
    }

    pub fn read_i64(&mut self) -> Result<i64, io::Error> {
        self.ensure(8)?;
        let bytes: [u8; 8] = self.data[self.pos..self.pos + 8].try_into().unwrap();
        self.pos += 8;
        Ok(if self.big_endian {
            i64::from_be_bytes(bytes)
        } else {
            i64::from_le_bytes(bytes)
        })
    }

    pub fn read_u64(&mut self) -> Result<u64, io::Error> {
        self.ensure(8)?;
        let bytes: [u8; 8] = self.data[self.pos..self.pos + 8].try_into().unwrap();
        self.pos += 8;
        Ok(if self.big_endian {
            u64::from_be_bytes(bytes)
        } else {
            u64::from_le_bytes(bytes)
        })
    }

    pub fn read_f64(&mut self) -> Result<f64, io::Error> {
        self.ensure(8)?;
        let bytes: [u8; 8] = self.data[self.pos..self.pos + 8].try_into().unwrap();
        self.pos += 8;
        Ok(if self.big_endian {
            f64::from_be_bytes(bytes)
        } else {
            f64::from_le_bytes(bytes)
        })
    }

    pub fn read_bool(&mut self) -> Result<bool, io::Error> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_cstring(&mut self) -> Result<String, io::Error> {
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }
        let bytes = &self.data[start..self.pos];
        if self.pos < self.data.len() {
            self.pos += 1; // skip the null terminator
        }
        String::from_utf8(bytes.to_vec()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    #[allow(dead_code)]
    pub fn read_string(&mut self) -> Result<String, io::Error> {
        let len = self.read_i32()?;
        if len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("negative string length: {len}"),
            ));
        }
        let bytes = self.read_bytes(len as usize)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, io::Error> {
        self.ensure(count)?;
        let val = self.data[self.pos..self.pos + count].to_vec();
        self.pos += count;
        Ok(val)
    }

    pub fn align(&mut self, n: usize) {
        let remainder = self.pos % n;
        if remainder != 0 {
            self.pos += n - remainder;
        }
    }

    fn ensure(&self, count: usize) -> Result<(), io::Error> {
        if self.data.len().saturating_sub(self.pos) < count {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read past end",
            ))
        } else {
            Ok(())
        }
    }
}
