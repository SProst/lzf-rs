use std;
use libc::{c_uint,c_void};

#[link(name = "lzf", kind = "static")]
extern {
    fn lzf_compress(in_data: *const c_void, in_len: c_uint, out_data: *const c_void, out_len: c_uint) -> c_uint;
    fn lzf_decompress(in_data: *const c_void, in_len: c_uint, out_data: *const c_void, out_len: c_uint) -> c_uint;
}

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum LzfError {
    BufferTooSmall,
    DataCorrupted,
    NoCompressionPossible,
    UnknownError(int)
}
pub type LzfResult<T> = Result<T, LzfError>;

/// Compress the given data, if possible.
/// The return value will be set to the error if compression fails.
///
/// The buffer is always set to the same size as the input buffer.
/// If that is not enough to hold the lzf-compressed data,
/// an error will be returned.
///
/// Example:
///
/// ```rust
/// let data = "foobar";
/// let compressed = lzf::compress(data.as_bytes());
/// ```
pub fn compress(data: &[u8]) -> LzfResult<Vec<u8>> {
    let data_len = data.len();
    let mut out : Vec<u8> = Vec::with_capacity(data_len);

    let result = unsafe { lzf_compress(data.as_ptr() as *const c_void, data_len as c_uint,
                                       out.as_ptr() as *const c_void, data_len as c_uint) };

    match result {
        0 => Err(NoCompressionPossible),
        _ => {
            unsafe { out.set_len(result as uint) };
            Ok(out)
        }
    }
}

/// Decompress the given data, if possible.
/// The return value will be set to the error if compression fails.
///
/// The length of the output buffer can be specified.
/// If the output buffer is not large enough to hold the decompressed data,
/// BufferTooSmall is returned.
/// Otherwise the number of decompressed bytes
/// (i.e. the original length of the data) is returned.
///
/// If an error in the compressed data is detected, DataCorrupted is returned.
///
/// Example:
///
/// ```rust,no_run
/// let data = "[your-compressed-data]";
/// let decompressed = lzf::decompress(data.as_bytes(), 10);
/// ```
pub fn decompress(data: &[u8], out_len: uint) -> LzfResult<Vec<u8>> {
    let mut out : Vec<u8> = Vec::with_capacity(out_len);

    let result = unsafe { lzf_decompress(data.as_ptr() as *const c_void, data.len() as c_uint,
                                         out.as_ptr() as *const c_void, out_len as c_uint) };
    match result {
        0 => {
            match std::os::errno() {
                7  => Err(BufferTooSmall),
                22 => Err(DataCorrupted),
                e  => Err(UnknownError(e))
            }
        },
        _ => {
            unsafe { out.set_len(result as uint) };
            Ok(out)
        }
    }
}

#[test]
fn test_compress_skips_short() {
    match compress("foo".as_bytes()) {
        Ok(_) => fail!("Compression did _something_, with is wrong for 'foo'"),
        Err(err) => assert_eq!(NoCompressionPossible, err)
    }
}

#[test]
fn test_compress_lorem() {
    let lorem = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.";

    match compress(lorem.as_bytes()) {
        Ok(compressed) => {
            assert_eq!(272, compressed.len())
        }
        Err(err) => fail!("Compression failed with error {}", err)
    }
}

#[test]
fn test_compress_decompress_lorem_round() {
    let lorem = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.";

    let compressed = match compress(lorem.as_bytes()) {
        Ok(c) => c,
        Err(err) => fail!("Compression failed with error {}", err)
    };

    match decompress(compressed.as_slice(), lorem.len()) {
        Ok(decompressed) => {
            assert_eq!(lorem.len(), decompressed.len());
            assert_eq!(lorem.as_bytes(), decompressed.as_slice());
        },
        Err(err) => fail!("Decompression failed with error {}", err)
    };
}

#[test]
fn test_decompress_fails_with_short_buffer() {
    let lorem = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.";

    let compressed = match compress(lorem.as_bytes()) {
        Ok(c) => c,
        Err(err) => fail!("Compression failed with error {}", err)
    };

    match decompress(compressed.as_slice(), 10) {
        Ok(_) => fail!("Decompression worked. That should not happen"),
        Err(err) => assert_eq!(BufferTooSmall, err)
    }
}

#[test]
fn test_decompress_fails_for_corrupted_data() {
    let lorem = "Lorem ipsum dolor sit amet";

    match decompress(lorem.as_bytes(), lorem.len()) {
        Ok(_) => fail!("Decompression worked. That should not happen"),
        Err(err) => assert_eq!(DataCorrupted, err)
    }
}