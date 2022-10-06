use std::mem::MaybeUninit;
use super::{BorrowedBuf, BufWriter, ErrorKind, Read, Result, Write, DEFAULT_BUF_SIZE, cursor};

/// Copies the entire contents of a reader into a writer
/// 
/// This function will contiuously read data from `reader` and then write it into `writer` in a streaming fashion until `reader` returns EoF
/// 
/// On success, the total number of bytes that were copied from `reader` to `writer` is returned.
/// 
/// # Errors
/// 
/// This function will return an error immediately if any call to [`read`] or [`write`] returns an error.
/// All instances of [`ErrorKind::Interrupted`] are handled by this function and the underlying operation is retried
/// 
/// [`read`]: Read::read
/// [`write`]: Write::wrie
pub fn copy<R, W>(reader: &mut R, writer: &mut W) -> Result<u64> 
where
    R : Read + ?Sized,
    W : Write + ?Sized
{
    // TODO(jel): how to handle platform specific optimizations?
    generic_copy(reader, writer)
}

/// The userspace read-write loop implementation of `io::copy` that is used when OS-specific specializations for copy offloading are not available or not applicable.
pub fn generic_copy<R, W>(reader: &mut R, writer: &mut W) -> Result<u64> 
where
    R : Read + ?Sized,
    W : Write + ?Sized,
{
    BufferedCopySpec::copy_to(reader, writer)
}

/// Specialization of the read-write loop that either uses a stack buffer or reuses the internal buffer of a `BufWriter`
trait BufferedCopySpec : Write {
    fn copy_to<R: Read + ?Sized>(reader: &mut R, writer: &mut Self) -> Result<u64>;
}

impl<W: Write + ?Sized> BufferedCopySpec for W {
    default fn copy_to<R: Read + ?Sized>(reader: &mut R, writer: &mut Self) -> Result<u64> {
        stack_buffer_copy(reader, writer)
    }
}

impl<W: Write> BufferedCopySpec for BufWriter<W> {
    fn copy_to<R: Read + ?Sized>(reader: &mut R, writer: &mut Self) -> Result<u64> {
        if writer.capacity() < DEFAULT_BUF_SIZE {
            return stack_buffer_copy(reader, writer);
        }

        let mut len = 0;
        let mut init = 0;

        loop {
            let buf = writer.buffer_mut();
            let mut read_buf : BorrowedBuf<'_> = buf.spare_capacity_mut().into();
        
            unsafe {
                // SAFETY: init is either 0 or the init_len from the previous interation
                read_buf.set_init(init);
            }

            if read_buf.capacity() >= DEFAULT_BUF_SIZE {
                let mut cursor = read_buf.unfilled();
                match reader.read_buf(cursor.reborrow()) {
                    Ok(()) => {
                        let bytes_read = cursor.written();

                        if bytes_read == 0 {
                            return Ok(len);
                        }

                        init = read_buf.init_len() - bytes_read;
                        len += bytes_read as u64;

                        // SAFETY: BorrowBuf guarantees all of its filled bytes are init
                        unsafe { buf.set_len(buf.len() + bytes_read) };

                        // Read again if the buffer still has enough capacity, as `BufWrtier` itself would do.
                        // This will occur if the reader returns short reads
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
                    Err(e) => return Err(e),
                }
            } else {
                writer.flush_buf()?;
                init = 0;
            }
        }
    }
}

fn stack_buffer_copy<R, W>(reader: &mut R, writer: &mut W) -> Result<u64>
where
    R : Read + ?Sized,
    W : Write + ?Sized,
{
    let buf : &mut[_] = &mut [MaybeUninit::uninit(); DEFAULT_BUF_SIZE];
    let mut buf : BorrowedBuf<'_> = buf.into();

    let mut len = 0;

    loop {
        match reader.read_buf(buf.unfilled()) {
            Ok(()) => {},
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }

        if buf.filled().is_empty() {
            break;
        }

        len += buf.filled().len() as u64;
        writer.write_all(buf.filled())?;
        buf.clear();
    }

    Ok(len)
}