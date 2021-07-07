pub enum ErrorMsg {
    Empty,
    Full,
}

pub enum RingBufferMode {
    Override=0,
    WriteNew
}

pub struct RingBuffer<T> {
    buffer: *mut T,
    capacity: isize,
    read_offset: usize,
    write_offset: usize,
    mode: RingBufferMode,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize, mode: RingBufferMode) -> Self {
        assert_ne!(capacity, 0);
        let align = std::mem::align_of::<T>();
        let element_size = std::mem::size_of::<T>();
        let layout = std::alloc::Layout::from_size_align(element_size * capacity, align)
            .expect("construction fail");
        let ptr = unsafe { std::alloc::alloc(layout) } as *mut T;

        RingBuffer {
            capacity: capacity as isize,
            buffer: ptr,
            read_offset: 0,
            write_offset: 0,
            mode: mode,
        }
    }

    fn overwrite(&mut self, element: T) {
        match self.mode {
            RingBufferMode::Override => {
                if self.is_full() {
                    let _ = self.read();
                }
            }
            RingBufferMode::WriteNew => {
                self.write(element);
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.read_offset == self.write_offset
    }

    fn is_full(&self) -> bool {
        self.write_offset - self.read_offset == self.capacity as usize
    }

    pub fn read(&mut self) -> Result<T, ErrorMsg> {
        if self.is_empty() {
            Err(ErrorMsg::Empty)
        } else {
            let value = unsafe {
                let read_ptr = self.buffer.offset(self.read_offset as isize);
                std::ptr::read(read_ptr)
            };

            self.read_offset += 1;
            Ok(value)
        }
    }

    pub fn write(&mut self, element: T) -> Result<(), ErrorMsg> {
        if self.is_full() {
            Err(ErrorMsg::Full)
        } else {
            unsafe {
                let write_ptr = self
                    .buffer
                    .offset((self.write_offset as isize) % (self.capacity as isize));
                std::ptr::write(write_ptr, element);
            }
            self.write_offset += 1;
            Ok(())
        }
    }
    // under construction
    // pub fn remove(&mut self, element: T) -> Result<(),ErrorMsg>{
    //     unsafe{
            
    //     }
    // }

    fn realign(&mut self) {
        if self.read_offset >= self.capacity as usize {
            self.read_offset -= self.capacity as usize;
            self.write_offset -= self.capacity as usize;
        }
    }

    pub fn clear(&mut self) {
        loop {
            match self.read() {
                Ok(_) => {}
                _ => break,
            }
            self.read_offset = 0;
            self.write_offset = 0;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_rb() {
        let mut rb = RingBuffer::new(10, RingBufferMode::Override);
        for i in 1..=10 {
            rb.write(i);
        }
        for i in 1..=10 {
            let val = rb.read().unwrap_or(0);
            assert_eq!(val, i);
        }

        let mut rb = RingBuffer::new(5, RingBufferMode::Override);
        for i in 1..=10 {
            rb.write(i);
        }
    }
}
