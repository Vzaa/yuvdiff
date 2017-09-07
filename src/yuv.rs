use std::u8;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::io::{Error, ErrorKind};
use std::fs::File;

pub struct Yuv {
    width: usize,
    height: usize,
    pix_y: Box<[u8]>,
    pix_u: Box<[u8]>,
    pix_v: Box<[u8]>,
}

// Pad 16 lines
pub fn buf_size_pad(w: usize, h: usize) -> usize {
    (w * h) + w * 16
}

pub fn buf_size(w: usize, h: usize) -> usize {
    (w * h)
}

impl Yuv {
    pub fn new(w: usize, h: usize) -> Yuv {
        let y_size = buf_size_pad(w, h);
        let uv_size = buf_size_pad(w / 2, h / 2);

        Yuv {
            width: w,
            height: h,
            pix_y: vec![0; y_size].into_boxed_slice(),
            pix_u: vec![0; uv_size].into_boxed_slice(),
            pix_v: vec![0; uv_size].into_boxed_slice(),
        }
    }

    fn frame_size(&self) -> usize {
        self.y_buf_size() + 2 * self.uv_buf_size()
    }

    fn y_buf_size(&self) -> usize {
        buf_size(self.width, self.height)
    }

    fn uv_buf_size(&self) -> usize {
        buf_size(self.width / 2, self.height / 2)
    }

    fn y_frame_mut(&mut self) -> &mut [u8] {
        let len = self.y_buf_size();
        &mut self.pix_y[0..len]
    }

    fn u_frame_mut(&mut self) -> &mut [u8] {
        let len = self.uv_buf_size();
        &mut self.pix_u[0..len]
    }

    fn v_frame_mut(&mut self) -> &mut [u8] {
        let len = self.uv_buf_size();
        &mut self.pix_v[0..len]
    }

    pub fn y_frame(&self) -> &[u8] {
        &self.pix_y[0..self.y_buf_size()]
    }

    pub fn y_frame_pad(&self) -> &[u8] {
        &self.pix_y
    }

    pub fn u_frame(&self) -> &[u8] {
        &self.pix_u[0..self.uv_buf_size()]
    }

    pub fn u_frame_pad(&self) -> &[u8] {
        &self.pix_u
    }

    pub fn v_frame(&self) -> &[u8] {
        &self.pix_v[0..self.uv_buf_size()]
    }

    pub fn v_frame_pad(&self) -> &[u8] {
        &self.pix_v
    }

    pub fn from_abs_diff(frame_a: &Yuv, frame_b: &Yuv) -> Result<Yuv, &'static str> {
        if !frame_a.same_size(frame_b) {
            return Err("Sizes differ");
        }

        let differ = |(a, b): (&u8, &u8)| (i16::from(*a) - i16::from(*b)).abs() as u8;

        let pix_y: Vec<_> = frame_a
            .pix_y
            .iter()
            .zip(frame_b.pix_y.iter())
            .map(&differ)
            .collect();
        let pix_u: Vec<_> = frame_a
            .pix_u
            .iter()
            .zip(frame_b.pix_u.iter())
            .map(&differ)
            .collect();
        let pix_v: Vec<_> = frame_a
            .pix_v
            .iter()
            .zip(frame_b.pix_v.iter())
            .map(&differ)
            .collect();

        Ok(Yuv {
            width: frame_a.width,
            height: frame_a.height,
            pix_y: pix_y.into_boxed_slice(),
            pix_u: pix_u.into_boxed_slice(),
            pix_v: pix_v.into_boxed_slice(),
        })
    }

    pub fn multiplied(mut self, m: u32) -> Yuv {
        let multiplier = |p: &u8| {
            let s: u32 = u32::from(*p) * m;
            if s >= u32::from(u8::MAX) {
                u8::MAX
            } else {
                s as u8
            }
        };

        for pix in self.pix_y.iter_mut() {
            *pix = multiplier(pix)
        }
        for pix in self.pix_u.iter_mut() {
            *pix = multiplier(pix)
        }
        for pix in self.pix_v.iter_mut() {
            *pix = multiplier(pix)
        }
        self
    }

    pub fn read<T: Read>(&mut self, reader: &mut T) -> io::Result<()> {
        reader.read_exact(self.y_frame_mut())?;
        reader.read_exact(self.u_frame_mut())?;
        reader.read_exact(self.v_frame_mut())?;
        Ok(())
    }

    pub fn same_size(&self, other: &Yuv) -> bool {
        self.width == other.width && self.height == other.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width_uv(&self) -> usize {
        self.width / 2
    }

    pub fn height_uv(&self) -> usize {
        self.height / 2
    }
}

pub struct YuvReader {
    frame: Yuv,
    reader: BufReader<File>,
    frame_cnt: u64,
    cursor: u64,
}

impl YuvReader {
    pub fn new(w: usize, h: usize, filename: &str) -> io::Result<YuvReader> {
        let file_in = File::open(filename)?;
        let reader = BufReader::new(file_in);
        Ok(YuvReader {
            frame: Yuv::new(w, h),
            reader: reader,
            frame_cnt: 0,
            cursor: 0,
        })
    }

    pub fn has_next(&self) -> bool {
        // panic if we can't get metadata
        let meta = self.reader.get_ref().metadata().unwrap();
        let remaning = meta.len() as i64 - self.cursor as i64;
        let fsize = self.frame.frame_size() as i64;
        remaning >= fsize
    }

    pub fn next_frame(&mut self) -> io::Result<()> {
        if self.has_next() {
            let fsize = self.frame.frame_size() as u64;
            self.frame.read(&mut self.reader)?;
            self.cursor += fsize;
            self.frame_cnt += 1;
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Not enough data in file",
            ))
        }
    }

    pub fn nth_frame(&mut self, n: u64) -> io::Result<()> {
        if n == 0 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Frame number needs to be >0",
            ));
        }

        // panic if we can't get metadata
        let meta = self.reader.get_ref().metadata().unwrap();
        let fsize = self.frame.frame_size() as u64;
        let cursor_loc: u64 = fsize * (n - 1);

        let remaning = meta.len() as i64 - cursor_loc as i64;

        if remaning > fsize as i64 {
            self.reader.seek(SeekFrom::Start(cursor_loc as u64))?;
            self.cursor = cursor_loc as u64;
            self.frame_cnt = n - 1;
            self.next_frame()
        } else {
            Err(Error::new(
                ErrorKind::UnexpectedEof,
                "Not enough data in file",
            ))
        }
    }

    pub fn prev_frame(&mut self) -> io::Result<()> {
        let n = self.frame_cnt - 1;
        self.nth_frame(n)
    }

    pub fn reset(&mut self) -> io::Result<()> {
        self.nth_frame(1)
    }

    pub fn cur_frame(&self) -> &Yuv {
        &self.frame
    }
}
