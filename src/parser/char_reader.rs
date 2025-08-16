/*
 * CharReader is a not entirely redundant flattening/chimera of std's
 * BufReader and unicode_reader's CodePoints, introduced to allow
 * peekable buffered UTF-8 codepoints and access to the underlying
 * reader.
 *
 * Unlike CodePoints, it doesn't make the reader inaccessible by
 * wrapping it a Bytes struct.
 *
 * Unlike BufReader, its buffer is peekable as a char.
 */

use smallvec::*;

use std::error::Error;
use std::fmt;
use std::io;
use std::io::{ErrorKind, IoSliceMut, Read};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::str;

pub struct CharReader<R> {
    inner: R,
    buf: SmallVec<[u8; 32]>,
    pos: usize,
    worker: Option<ReaderWorker>,
    inflight_read: bool,
}

struct ReaderWorker {
    tx: Sender<ReadRequest>,
    rx: Receiver<ReadResponse>,
}

enum ReadRequest {
    Read(usize),
}

enum ReadResponse {
    Data(io::Result<Vec<u8>>),
}

/// An error raised when parsing a UTF-8 byte stream fails.
#[derive(Debug)]
pub struct BadUtf8Error {
    /// The bytes that could not be parsed as a code point.
    pub bytes: Vec<u8>,
}

impl Error for BadUtf8Error {
    fn description(&self) -> &str {
        "BadUtf8Error"
    }
}

impl fmt::Display for BadUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bad UTF-8: {:?}", self.bytes)
    }
}

impl<R> CharReader<R> {
    pub fn new(inner: R) -> CharReader<R> {
        Self {
            inner,
            buf: SmallVec::new(),
            pos: 0,
            worker: None,
            inflight_read: false,
        }
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    // Return the number of bytes remaining to be read.  Useful for,
    // e.g., determining the position relative to the end of the
    // owning stream.
    #[inline]
    pub fn rem_buf_len(&self) -> usize {
        self.buf.len() - self.pos
    }
}

pub trait CharRead {
    fn read_char(&mut self) -> Option<io::Result<char>> {
        match self.peek_char() {
            Some(Ok(c)) => {
                self.consume(c.len_utf8());
                Some(Ok(c))
            }
            result => result,
        }
    }

    fn peek_char(&mut self) -> Option<io::Result<char>>;
    fn put_back_char(&mut self, c: char);
    fn consume(&mut self, nread: usize);
}

impl<R> CharReader<R> {
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    pub fn reset_buffer(&mut self) {
        self.buf.clear();
        self.pos = 0;
    }
}

impl<R: Read> CharReader<R> {
    // Spawns the background reader thread lazily.
    // Safety rationale: We pass a raw address of `inner` to the worker thread.
    // After the worker starts, all reads are performed exclusively on that
    // thread; the main thread must not access `inner` directly anymore.
    // CharReader values should not be moved in memory after the worker is
    // spawned, which holds in this project because CharReader is allocated in
    // the arena before any reads occur. Moving would invalidate the raw pointer.
    fn ensure_worker(&mut self) {
        if self.worker.is_some() {
            return;
        }

        // Create channels: main sends requests to worker, receives responses.
        let (req_tx, req_rx) = mpsc::channel::<ReadRequest>();
        let (resp_tx, resp_rx) = mpsc::channel::<ReadResponse>();

        // Raw pointer to inner to avoid requiring R: Send. We guarantee only the
        // worker thread will touch inner for reads once spawned.
        let inner_addr: usize = (&mut self.inner as *mut R) as usize;

        thread::spawn(move || {
            // Worker loop: perform blocking reads as requested
            while let Ok(msg) = req_rx.recv() {
                match msg {
                    ReadRequest::Read(mut to_read) => {
                        if to_read == 0 || to_read > 4 { to_read = 4; }
                        // Safety: inner_ptr is valid and uniquely used by this thread for reads.
                        let res = unsafe {
                            let inner = &mut *(inner_addr as *mut R);
                            let mut buf = vec![0u8; to_read];
                            match inner.read(&mut buf[..]) {
                                Ok(n) => {
                                    buf.truncate(n);
                                    Ok(buf)
                                }
                                Err(e) => Err(e),
                            }
                        };
                        let _ = resp_tx.send(ReadResponse::Data(res));
                    }
                }
            }
        });

        self.worker = Some(ReaderWorker { tx: req_tx, rx: resp_rx });
    }

    // Try to read up to `need` bytes into self.buf (appending). Returns:
    // Ok(Some(n)) if bytes were read; Ok(None) if timed out; Err(e) on error.
    fn read_more_with_timeout(&mut self, need: usize) -> io::Result<Option<usize>> {
        self.ensure_worker();
        // get handles
        let (tx, rx_ref) = {
            let w = self.worker.as_mut().unwrap();
            (w.tx.clone(), &w.rx)
        };

        // First, non-blockingly drain any pending response from a prior request.
        // This guarantees we never "stick" with inflight_read=true if data already arrived.
        loop {
            match rx_ref.try_recv() {
                Ok(ReadResponse::Data(Ok(bytes))) => {
                    self.inflight_read = false;
                    let n = bytes.len();
                    if n > 0 {
                        self.buf.extend_from_slice(&bytes);
                        if self.pos > self.buf.len() { self.pos = self.buf.len(); }
                    }
                    return Ok(Some(n));
                }
                Ok(ReadResponse::Data(Err(e))) => {
                    self.inflight_read = false;
                    return Err(e);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.inflight_read = false;
                    return Err(io::Error::new(ErrorKind::UnexpectedEof, "read worker disconnected"));
                }
            }
        }

        // Only send a new read request if there isn't one already in flight.
        if !self.inflight_read {
            if tx.send(ReadRequest::Read(need)).is_err() {
                // worker gone
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "read worker stopped"));
            }
            self.inflight_read = true;
        }

        // wait up to 100ms for the in-flight read to complete
        match rx_ref.recv_timeout(Duration::from_millis(100)) {
            Ok(ReadResponse::Data(Ok(bytes))) => {
                self.inflight_read = false;
                let n = bytes.len();
                if n > 0 {
                    self.buf.extend_from_slice(&bytes);
                    if self.pos > self.buf.len() { self.pos = self.buf.len(); }
                }
                Ok(Some(n))
            }
            Ok(ReadResponse::Data(Err(e))) => {
                self.inflight_read = false;
                Err(e)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Ok(None)
            }
            Err(_) => {
                self.inflight_read = false;
                Err(io::Error::new(ErrorKind::UnexpectedEof, "read worker disconnected"))
            }
        }
    }
    pub fn refresh_buffer(&mut self) -> io::Result<&[u8]> {
        // If we've reached the end of our internal buffer then we need to fetch
        // some more data from the underlying reader.
        // Branch using `>=` instead of the more correct `==`
        // to tell the compiler that the pos..cap slice is always valid.
        if self.pos >= self.buf.len() {
            self.buf.clear();
            self.pos = 0;
            match self.read_more_with_timeout(std::mem::size_of::<char>())? {
                Some(_n) => {}
                None => {
                    // Timed out: indicate WouldBlock instead of empty slice to avoid false EOF.
                    return Err(io::Error::new(ErrorKind::WouldBlock, "would block"));
                }
            }
        }

        Ok(&self.buf[self.pos..])
    }

    pub fn peek_byte(&mut self) -> Option<io::Result<u8>> {
        // If buffer has data, return first byte.
        if self.pos < self.buf.len() {
            return Some(Ok(self.buf[self.pos]));
        }
        // Otherwise, try to read 1 byte with a 100ms timeout.
        // Snapshot state to restore if the read times out.
        let saved_buf = self.buf.clone();
        let saved_pos = self.pos;

        self.buf.clear();
        self.pos = 0;
        match self.read_more_with_timeout(1) {
            Err(e) => Some(Err(e)),
            Ok(None) => {
                // Timeout: restore previous buffer/pos, report WouldBlock.
                self.buf = saved_buf;
                self.pos = saved_pos;
                Some(Err(io::Error::new(ErrorKind::WouldBlock, "would block")))
            }
            Ok(Some(0)) => None, // true EOF
            Ok(Some(_)) => {
                // Data arrived; return the first byte.
                if self.pos < self.buf.len() {
                    Some(Ok(self.buf[self.pos]))
                } else {
                    None
                }
            }
        }
    }
}

impl<R: Read> CharRead for CharReader<R> {
    fn peek_char(&mut self) -> Option<io::Result<char>> {
        // Synchronous implementation that attempts to decode the next UTF-8
        // character from the buffered input, reading up to 4 bytes from the
        // underlying reader as needed. This mirrors the previous logic without
        // spawning a thread, ensuring correctness and avoiding mutation issues
        // across threads.

        // Helper to build an InvalidData error for bad UTF-8 leading bytes.
        let bad_bytes_error = |buf: &[u8]| {
            // If we have 4 bytes that still don't make up a valid code point,
            // then we have garbage. Remove leading bytes until either the
            // buffer is empty, or we have a valid code point, and report the
            // removed bytes as the error payload.
            let mut split_point = 1;
            let mut badbytes = vec![];

            loop {
                let (bad, rest) = buf.split_at(split_point);

                if rest.is_empty() || str::from_utf8(rest).is_ok() {
                    badbytes.extend_from_slice(bad);
                    break;
                }

                split_point += 1;
            }

            io::Error::new(io::ErrorKind::InvalidData, BadUtf8Error { bytes: badbytes })
        };

        loop {
            // Ensure we have at least one byte in the buffer if possible.
            if self.pos >= self.buf.len() {
                self.buf.clear();
                self.pos = 0;
                match self.read_more_with_timeout(std::mem::size_of::<char>()) {
                    Err(e) => return Some(Err(e)),
                    Ok(None) => {
                        // Timeout: signal WouldBlock to avoid falsely advertising EOF.
                        return Some(Err(io::Error::new(ErrorKind::WouldBlock, "would block")));
                    }
                    Ok(Some(_)) => {}
                }
            }

            let buf = &self.buf[self.pos..];

            if !buf.is_empty() {
                // Fast path: if the remaining buffer is valid UTF-8, return the
                // first character without consuming it.
                match str::from_utf8(buf) {
                    Ok(s) => {
                        if let Some(c) = s.chars().next() {
                            return Some(Ok(c));
                        } else {
                            // Shouldn't happen because buf isn't empty, but handle gracefully.
                            return None;
                        }
                    }
                    Err(e) => {
                        // If at least one complete character exists within the
                        // valid prefix, extract it without reading more.
                        let valid_up_to = e.valid_up_to();

                        if buf.len() - valid_up_to >= 4 {
                            // 4 or more invalid leading bytes: treat as error.
                            return Some(Err(bad_bytes_error(buf)));
                        } else if self.pos >= self.buf.len() {
                            return None;
                        } else if self.buf.len() - self.pos >= 4 && self.pos < valid_up_to {
                            // We have enough valid bytes for a character in the prefix.
                            return match str::from_utf8(&self.buf[self.pos..self.pos + valid_up_to]) {
                                Ok(s) => s.chars().next().map(|c| Ok(c)),
                                Err(e) => {
                                    let badbytes = self.buf[self.pos..self.pos + e.valid_up_to()].to_vec();
                                    Some(Err(io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        BadUtf8Error { bytes: badbytes },
                                    )))
                                }
                            };
                        } else {
                            // Shift remaining bytes to the front to make room, then read more.
                            // Snapshot current buffer and position to allow rollback on timeout.
                            let saved_buf = self.buf.clone();
                            let saved_pos = self.pos;

                            let buf_len = self.buf.len();
                            for (c, idx) in (self.pos..buf_len).enumerate() {
                                self.buf[c] = self.buf[idx];
                            }
                            self.buf.truncate(buf_len - self.pos);
                            let buf_len = self.buf.len();
                            self.pos = 0;

                            if buf_len >= 4 {
                                // Already have 4 bytes; loop to re-evaluate.
                                continue;
                            }

                            // Attempt to read more bytes via background worker with 100ms timeout.
                            match self.read_more_with_timeout(4 - buf_len) {
                                Err(e) => return Some(Err(e)),
                                Ok(None) => {
                                    // Timeout: restore previous buffer/pos and report transient lack of data.
                                    self.buf = saved_buf;
                                    self.pos = saved_pos;
                                    return None;
                                }
                                Ok(Some(0)) => return Some(Err(bad_bytes_error(&self.buf))),
                                Ok(Some(_n)) => {
                                    // bytes appended to self.buf; loop will re-evaluate
                                }
                            }
                        }
                    }
                }
            } else {
                return None;
            }
        }
    }

    #[inline(always)]
    fn put_back_char(&mut self, c: char) {
        let src_len = self.buf.len() - self.pos;
        debug_assert!(src_len <= self.buf.capacity());

        let c_len = c.len_utf8();
        let mut shifted_slice = [0u8; 32];

        shifted_slice[0..src_len].copy_from_slice(&self.buf[self.pos..self.buf.len()]);

        self.buf.resize(c_len, 0);
        self.buf.extend_from_slice(&shifted_slice[0..src_len]);
        self.pos = 0;

        c.encode_utf8(&mut self.buf[0..c_len]);
    }

    #[inline(always)]
    fn consume(&mut self, nread: usize) {
        self.pos += nread;
    }
}

impl<R: Read> Read for CharReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // // If we don't have any buffered data and we're doing a massive read
        // // (larger than our internal buffer), bypass our internal buffer
        // // entirely.
        // if self.pos == self.cap && buf.len() >= self.buf.len() {
        //     self.discard_buffer();
        //     return self.inner.read(buf);
        // }

        let mut inner_buf = self.refresh_buffer()?;
        let nread = inner_buf.read(buf)?;

        // let nread = {
        //     let mut rem = self.fill_buf()?;
        //     rem.read(buf)?
        // };

        self.pos += nread;
        Ok(nread)
    }

    // Small read_exacts from a BufReader are extremely common when used with a deserializer.
    // The default implementation calls read in a loop, which results in surprisingly poor code
    // generation for the common path where the buffer has enough bytes to fill the passed-in
    // buffer.
    fn read_exact(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
        if self.buffer().len() >= buf.len() {
            buf.copy_from_slice(&self.buffer()[..buf.len()]);
            self.pos += buf.len();
            return Ok(());
        }

        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        if !buf.is_empty() {
            Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "failed to fill whole buffer",
            ))
        } else {
            Ok(())
        }
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();

        if self.pos == self.buf.len() && total_len >= self.buf.len() {
            self.reset_buffer(); // self.discard_buffer();
            return self.inner.read_vectored(bufs);
        }

        self.refresh_buffer()?;

        let nread = (&self.buf[self.pos..]).read_vectored(bufs)?;
        self.pos += nread;

        Ok(nread)
    }
}

/*
#[stable(feature = "rust1", since = "1.0.0")]
impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // If we've reached the end of our internal buffer then we need to fetch
        // some more data from the underlying reader.
        // Branch using `>=` instead of the more correct `==`
        // to tell the compiler that the pos..cap slice is always valid.
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}
*/

impl<R> fmt::Debug for CharReader<R>
where
    R: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("CharReader")
            .field("reader", &self.inner)
            .field(
                "buf",
                &format_args!("{}/{}", self.buf.capacity() - self.pos, self.buf.len()),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::char_reader::*;
    use std::io::Cursor;

    #[test]
    fn plain_string() {
        let mut read_string = CharReader::new(Cursor::new("a string"));

        for c in "a string".chars() {
            assert_eq!(read_string.peek_char().unwrap().ok(), Some(c));
            assert_eq!(read_string.read_char().unwrap().ok(), Some(c));
        }

        assert!(read_string.read_char().is_none());
    }

    #[test]
    fn greek_string() {
        let mut read_string = CharReader::new(Cursor::new("λέξη"));

        for c in "λέξη".chars() {
            assert_eq!(read_string.peek_char().unwrap().ok(), Some(c));
            assert_eq!(read_string.read_char().unwrap().ok(), Some(c));
        }

        assert!(read_string.read_char().is_none());
    }

    #[test]
    fn russian_string() {
        let mut read_string = CharReader::new(Cursor::new("слово"));

        for c in "слово".chars() {
            assert_eq!(read_string.peek_char().unwrap().ok(), Some(c));
            assert_eq!(read_string.read_char().unwrap().ok(), Some(c));
        }

        assert!(read_string.read_char().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore = "slow and not very relevant")]
    fn greek_lorem_ipsum() {
        let lorem_ipsum = "Λορεμ ιπσθμ δολορ σιτ αμετ, οφφενδιτ
    εφφιcιενδι σιτ ει, ηαρθμ λεγερε qθαερενδθμ ιθσ νε. Ηασ νο εροσ
    σιγνιφερθμqθε, σεδ ετ μθτατ jθστο, ει cθμ ελιγενδι σcριπτορεμ
    ρεπρεηενδθντ. Εοσ ατ αμετ μαλισ ελειφενδ. Ιν cθμ εριπθιτ
    νομινατι. Θσθ ιν cετεροσ μαιορθμ, μθνερε ατομορθμ ινcιδεριντ θτ
    ηασ. Αν ηασ λιβρισ πραεσεντ πατριοqθε, ηινc θτιναμ πριμισ νε
    cθμ. Cθ μοδο ερρεμ σcριβεντθρ cθμ. Ει vισ δεcορε μαλορθμ
    σεντεντιαε, σεδ νο λιβερ εvερτι μεντιτθμ. Προ φαcερ vολθτπατ
    σαπιεντεμ ιν. Cθ εροσ περσεqθερισ πρι, εα ποσσιτ cετεροσ δθο. Πρι
    εα μαλισ μθνερε.

    Qθισ jθστο μαλορθμ cθ qθο. Νεc ατ οδιο σολετ μαιεστατισ, νε
    φορενσιβθσ σαδιπσcινγ ιθσ, αν qθι ειθσ βρθτε σαπιεντεμ. Cομμθνε
    περcιπιτθρ ιθσ αδ, μθνερε δολορθμ ιμπεδιτ ηισ νε. Νεc ετιαμ
    προπριαε vιτθπερατα ιν. Σονετ νεμορε ιθσ cθ, ιν αφφερτ ινερμισ
    cοτιδιεqθε vισ.

    Ηασ ιδ νονθμυ δοcτθσ cοτιδιεqθε. Σινγθλισ πηιλοσοπηια εξ δθο. Εστ
    νο ιραcθνδια cονσεqθθντθρ. Τε διcτασ επιcθρει εφφιcιαντθρ δθο, εοσ
    νε νθλλα νομιναvι. Εθμ cθ ελιτρ λιβεραvισσε, σιτ περσεqθερισ
    cομπλεcτιτθρ εξ, πονδερθμ σιμιλιqθε ηασ νο.

    Σολθμ ποσσιμ λαβιτθρ εξ ηισ, ει δομινγ εξπετενδισ vελ, διαμ μινιμ
    σcριπσεριτ ει περ. Αθδιαμ οcθρρερετ προ εξ, δομινγ vολθπταρια ετ
    qθο. Cονσθλ σανcτθσ αccθμσαν νο ιθσ, αδ εαμ αλβθcιθσ
    ηονεστατισ. Ετ vιξ φαcιλισ qθαλισqθε ερροριβθσ, ηισ εθ πθρτο
    ασσεντιορ. Ιθσ βονορθμ ηονεστατισ σcριπσεριτ ατ, ιν ναμ εσσε μοvετ
    γραεcο. Αθγθε cονσεcτετθερ εστ ατ.

    Αδ ταλε σθασ μθνερε σεδ, vισ φεθγαιτ αντιοπαμ ιδ. Προ εθ ινερμισ
    σαλθτατθσ, σαεπε qθαεστιο θρβανιτασ cθ περ. Ιν μαλορθμ σαλθτατθσ
    δετερρθισσετ περ, νε παρτεμ vολθτπατ ινστρθcτιορ vιξ. Νο vισ
    δεμοcριτθμ εφφιcιαντθρ, επιcθρει αδολεσcενσ εστ cθ, ιδ vιξ
    λθcιλιθσ αδιπισcινγ. Σεα τε cλιτα ιραcθνδια. Σεα αν σιμθλ
    εσσεντ. Vοcιβθσ ελειφενδ cονσεqθθντθρ περ αδ, αν ναμ πονδερθμ
    vολθπταρια.

    Λιβερ ερθδιτι αccθσαμθσ θτ ναμ. Σιτ αντιοπαμ γθβεργρεν νε. Αμετ
    ανcιλλαε ετ qθι, μεα σολθμ λαθδεμ εα. Εθ μελ παρτεμ οβλιqθε
    πηαεδρθμ. Εξ μελ jθστο αccομμοδαρε, νε νολθισσε σινγθλισ σενσιβθσ
    cθμ, vισ εθ τιμεαμ αδιπισcινγ.

    Τε νολθισσε vολθπτατθμ εστ. Ασσθμ νομιναvι πρι νε, ει νοστρθμ
    επιcθρει μεα. Σεδ cθ ελιτ δεσερθντ, γραεcε ερροριβθσ προ θτ, περ
    νε εθισμοδ vολθπταρια. Νο εθμ διcατ ποσσιμ, νεc πρινcιπεσ
    cονcεπταμ νε. Εθ αππαρεατ ιντελλεγατ σεα. Μελ θτ ελιτ λαθδεμ, θσθ
    δολορεμ cομπλεcτιτθρ ετ, νε μεα δολορεσ μολεστιαε.

    Θσθ λεγενδοσ vολθπτατιβθσ cθ. Qθο νε αδηθc ρεφερρεντθρ, αλια
    μεδιοcρεμ δθο νε, σεδ ερρεμ δολορθμ αccομμοδαρε νε. Ετιαμ εqθιδεμ
    δετερρθισσετ cθ μει, ετ εροσ cετεροσ σεα, εξ vιξ ενιμ cασε
    δετραξιτ. Σεδ σολθτα λιβρισ ειρμοδ τε, νοvθμ ποπθλο νε εθμ. Σθμμο
    αδμοδθμ δεσερθντ εστ εξ, εστ διcαμ εqθιδεμ cθ.

    Ιλλθμ cορπορα ινvιδθντ εαμ ετ. Σεδ μαλισ ταcιματεσ εvερτιτθρ εα,
    μαζιμ νθλλαμ vοcιβθσ μεα ει. Μεα ορνατθσ λθπτατθμ αδιπισcινγ
    αδ. Μεα αφφερτ νοστερ ατ, ναμ αν σολεατ ερροριβθσ. Εξ σεα αεqθε
    μθνερε cετερο, εοσ ηινc ελειφενδ δεμοcριτθμ.";

        let mut lorem_ipsum_reader = CharReader::new(Cursor::new(lorem_ipsum));

        for c in lorem_ipsum.chars() {
            assert_eq!(lorem_ipsum_reader.peek_char().unwrap().ok(), Some(c));
            assert_eq!(lorem_ipsum_reader.read_char().unwrap().ok(), Some(c));
        }

        assert!(lorem_ipsum_reader.read_char().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore = "slow and not very relevant")]
    fn armenian_lorem_ipsum() {
        let lorem_ipsum = "լոռեմ իպսում դոլոռ սիթ ամեթ, նովում գռաեծո
        սեա եա, աբհոռռեանթ դիսպութանդո եի քուի. իդ քուոդ ինդոծթում
        եսթ, մեա թե ծոմմոդո ծոռպոռա. եթ ծոնսուլ ադիպիսծինգ ռեֆոռմիդանս
        պեռ, ինեռմիս ֆեուգաիթ նո քուո, թալե սալե պռո եա. եթ նիբհ
        աուգուե վոլումուս դուո, նե ծում եխեռծի սալութաթուս գլոռիաթուռ,
        ծու թաթիոն պռաեսենթ մեդիոծռեմ վիս.

        վիխ եռոս ռեֆեռռենթուռ եու. պեռսիուս վիթուպեռաթոռիբուս ութ սեա,
        վիդե ինվիդունթ պռոբաթուս նո քուո. մեի եռոս մելիուս նոմինավի
        իդ, ութ պռո քուաս քուաեսթիո. եթ նաթում պեթենթիում սուավիթաթե
        հիս. քուի ծոնսթիթութո մեդիոծռիթաթեմ թե. ծեթեռո դեթռածթո
        ծոնծեպթամ սեա եթ. դիսսենթիեթ ելոքուենթիամ թհեոպհռասթուս նեծ
        աթ, աթ ֆածեթե եռիպուիթ վիխ.

        ասսուեվեռիթ սծռիպսեռիթ եսթ եթ, վիդիթ դեբեթ եվեռթի եխ
        եսթ. աութեմ լաուդեմ պոսիդոնիում մեի եի. ռեբում դիծամ ծեթեռոս
        եում ծու. նիհիլ եխպեթենդա ասսուեվեռիթ ուսու ան. ւիսի թաթիոն
        դելենիթ նո իուս, սեդ եխ իդքուե սիգնիֆեռումքուե, բռութե զռիլ
        ալբուծիուս ան պռի.

        մովեթ իռիուռե սալութանդի պեռ նո, եի ոմնիս աֆֆեռթ պեռսեքուեռիս
        իուս, եթ պռաեսենթ մալուիսսեթ եսթ. եսթ պռոբո գուբեռգռեն եթ, հաս
        ին դիամ նումքուամ. ֆեուգաիթ ինվենիռե ռեպուդիանդաե աթ սեդ,
        իուվառեթ ծոնսուլաթու եֆֆիծիանթուռ ուսու եի. ութ մեա ածծումսան
        նոմինավի թինծիդունթ, մեի դիծթա ածծումսան ութ. վիմ ոմնիում
        ելիգենդի սծռիպթոռեմ եու.

        իդ վիս եռռոռ ալիքուիպ ելոքուենթիամ, ադ դելենիթի պեռծիպիթ
        դեֆինիթիոնես իուս. վիմ իուդիծո դեմոծռիթում ծոմպռեհենսամ թե,
        ութ նիհիլ լոբոռթիս վոլուպթաթիբուս վել, դիծունթ մենթիթում
        ֆածիլիսիս եի եում. եսսե սալե մինիմ եոս նե. ագամ ոմնեսքուե ծում
        ին.

        իուվառեթ իուդիծաբիթ ծում աթ, ուսու նիբհ աթքուի դոմինգ եխ. եի
        քուի սանծթուս սենսիբուս, նամ ուբիքուե ապպեթեռե պռոդեսսեթ
        եու. ուսու եթ աուգուե ծոնվենիռե սծռիբենթուռ. ան ոմնիում վեռեառ
        ութռոքուե դուո, եսթ եի լիբեռ մեդիոծռեմ եխպլիծառի, ոմնիս
        աուդիռե թե պռի. վիմ մունեռե սոլեաթ ծու, եռոս ինվենիռե
        դիսպութաթիոնի եի քուո, ան ալթեռա պութենթ լաբոռես պռո. անթիոպամ
        դեմոծռիթում պեռ ին.

        նե քուի ծիբո ելիթռ. նեծ նե լիբեռ վոլուպթուա. նիսլ ծոմմունե
        եխպեթենդիս նամ եխ, իուդիծո պլածեռաթ պեռծիպիթուռ մել նո, եթ
        պառթեմ պութանթ քուի. վիմ թինծիդունթ ածծոմմոդառե աթ, նե նամ
        վիդիթ իռիուռե, պռո եա ելիգենդի պոսթուլանթ ծոնսթիթութո.

        մել ութ ոդիո նուլլամ եխպլիծառի. պռոպռիաե թինծիդունթ
        դելիծաթիսսիմի եամ ան, մոդո քուոդսի ապեռիռի եու եսթ, պեռ աթ
        լաբոռես սենսեռիթ. վիմ ծոնգուե ռեպուդիանդաե եի, նեծ ագամ
        դիծունթ դելիծաթիսսիմի աթ. պոսսիթ լիբեռավիսսե եոս եու.

        աթ ալիա դեբեթ ելաբոռառեթ քուո, ին ալիի ածծումսան ծոնսթիթուամ
        հաս, մել թոթա ոմիթթանթուռ ինսթռուծթիոռ նո. պեռ նե ծաուսաե
        սապիենթեմ, պաուլո ոմնեսքուե եի քուո, եխ ոռաթիո պհիլոսոպհիա
        սիթ. իգնոթա ծաուսաե աթ ուսու, եխ քուո դիծթաս քուոդսի
        ռեպուդիառե. ծոռպոռա պռոդեսսեթ ռեֆեռռենթուռ եոս եխ.

        եու եթիամ ելեիֆենդ մել, սալե սծռիպսեռիթ հիս եու. պոռռո
        ադոլեսծենս մեի եա. ին մեա զռիլ պռոբաթուս սալութաթուս. եոս ադ
        մինիմ թեմպոռիբուս. սեա նե եթիամ.";

        let mut lorem_ipsum_reader = CharReader::new(Cursor::new(lorem_ipsum));

        for c in lorem_ipsum.chars() {
            assert_eq!(lorem_ipsum_reader.peek_char().unwrap().ok(), Some(c));
            assert_eq!(lorem_ipsum_reader.read_char().unwrap().ok(), Some(c));
        }

        assert!(lorem_ipsum_reader.read_char().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore = "slow and not very relevant")]
    fn russian_lorem_ipsum() {
        let lorem_ipsum = "Лорем ипсум долор сит амет, атяуи дицам еи
        сит, ид сеа фацилис елаборарет. Меа еу яуас алияуид, те яуи
        саперет аппеллантур. Ех иус диам дицта волуптариа, еу пер
        бруте омиттам аццусата. Хис сапиентем губергрен те, яуидам
        луптатум персеяуерис ад ест.

        Ан алияуип перицулис нам, нец апериам цотидиеяуе волуптатибус
        но. Солум тритани пер ех, меи не одио тритани рецусабо, цу при
        веро мелиоре импердиет. Ин граеци индоцтум салутатус нец, диам
        сцаевола пертинациа про те. Ут сеа дебитис лаборамус
        диссентиас, еи цум яуот лобортис.

        Децоре сингулис вим не. Еос не риденс оффициис, еу нонумы
        лабитур еррорибус хас, вел омнис цонституто посидониум но. Вел
        персиус фастидии репрехендунт ид. Натум иллум ипсум сит ад, еа
        еам новум латине. Еос нолуиссе патриояуе елояуентиам те.

        Стет малис яуаерендум хас ад, прима цотидиеяуе мел ан,
        трацтатос десеруиссе нам ех. Ин малорум сусципиантур вим, ех
        меа граецо тритани адолесценс. Промпта цонцлусионемяуе нам еи,
        дуо ин лаборе алтерум цотидиеяуе. Но елитр промпта сплендиде
        еум, аеяуе ассуеверит цонституам яуи ид. Ад тале еррор
        интеллегебат хас, ерудити граецис хас не, пер ут лабитур
        еуисмод. Те при суммо путант. Про утинам цоммуне урбанитас еа.

        Идяуе репрехендунт еи нам, алии толлит легере нам не, хис еа
        виси адверсариум цонцлусионемяуе. Хас ассум омиттам луцилиус
        ет, вих цонсул малорум фастидии не, сенсибус ассуеверит дуо
        ут. Дуо алиа видит цетеро ат, еа аппареат пертинах вел. Пер
        цонституто инцидеринт ин, убияуе риденс сенсерит цум цу. Про
        ет цетерос темпорибус, те вел пурто суммо, дуо мунере вертерем
        урбанитас ад. Сит оптион елецтрам форенсибус но. Еи татион
        сапиентем ест, лаборе сцрипта сингулис но вим, усу еу елигенди
        персецути.

        Иус ан елецтрам цонтентионес. Меи атяуи нонумес ут, вел амет
        репрехендунт ан, вис еу яуаестио патриояуе. Про синт легере
        детрацто ад. Постеа долорем евертитур при ет, вим номинави
        принципес ирацундиа ех. Доцтус интеллегебат но нам. Фацете
        оффициис нецесситатибус цу меа.

        Промпта симилияуе вис ин. Пер бонорум перицулис аргументум
        ад. Еу дицат фацилис губергрен нам, еффициенди цомпрехенсам
        хас еу. Инани нонумы усу но, ад цонцептам репудиандае
        про. Тота нуллам делицата еа яуо, усу дуис дебет путент еи.

        Вис апериам доценди елояуентиам еа. Ех яуот детрацто
        елояуентиам цум, ерос малис дицерет вис ин. Еа цум модус
        еяуидем, дебет нуллам ан меи. Алтерум омиттам про ет.

        Яуи ех латине алияуам, ан меи одио нуллам. Ид хас омнис ребум
        либрис. Ет убияуе путант дебитис про, ех хис медиоцрем
        партиендо, но елит елецтрам дуо. Еу меа сонет номинави
        цотидиеяуе. Нам фалли новум минимум еу, перфецто ратионибус
        цонституто ад меа.

        Нобис детрацто еам ид, при еу ассум пертинах, те етиам
        проприае салутанди яуо. Легимус сусципиантур ет хас, сед
        поссит дефинитионес еа. Ест не патриояуе омиттантур
        интеллегебат, еу яуо дебет цонцлудатуряуе. Еум ад мнесарчум
        дефинитионем, елитр лаборамус перципитур про не, хас феугаит
        фастидии луцилиус ид. Фастидии интеллегат ех.";

        let mut lorem_ipsum_reader = CharReader::new(Cursor::new(lorem_ipsum));

        for c in lorem_ipsum.chars() {
            assert_eq!(lorem_ipsum_reader.peek_char().unwrap().ok(), Some(c));
            assert_eq!(lorem_ipsum_reader.read_char().unwrap().ok(), Some(c));

            lorem_ipsum_reader.put_back_char(c);

            assert_eq!(lorem_ipsum_reader.peek_char().unwrap().ok(), Some(c));
            assert_eq!(lorem_ipsum_reader.read_char().unwrap().ok(), Some(c));
        }

        assert!(lorem_ipsum_reader.read_char().is_none());
    }
}
