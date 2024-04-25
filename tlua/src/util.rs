use std::future::Future;

// METHODS
pub fn static_ref<T>(val: T) -> &'static T {
    Box::leak(Box::new(val))
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    tokio::runtime::Runtime::new().unwrap().block_on(future)
}

// EXTENSION TRAITS

pub struct TextError(pub String);

pub trait ResultExt<T, E> {
    fn map_text_err(self, msg: &str) -> Result<T, TextError>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn map_text_err(self, msg: &str) -> Result<T, TextError> {
        self.map_err(|_| TextError(msg.to_string()))
    }
}

// EXTERNAL TRAITS
impl From<TextError> for mlua::Error {
    fn from(err: TextError) -> mlua::Error {
        mlua::Error::external(err.0)
    }
}

pub mod streams {
    //! Utilities for reading from streams asynchronously.
    //!
    //! Most functions here return `io::Result<Option<T>>`. That should
    //! be understood as follows:
    //! - `Ok(Some(T))` means that the value was successfully read.
    //! - `Ok(None)` means that the value was not found. We catch for `EOF`.
    //! - `Err(err)` means that an error occurred, except `EOF`.
    //!
    //! You can also notice that some functions have a `PhantomData<T>` in a return
    //! type. This is to ensure that the type `T` is not actually constructed.
    //! But we can encode the type `T` in the return type, so that the caller
    //! can use it to parse the type `T` if needed. Pretty neat, huh?

    use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};

    use crate::check_eof;

    // advance past float
    // look for [+-]{0,1}[0-9]+ // int
    // then optional .[0-9]+
    // then optional e[+-]{0,1}[0-9]+ // `e` followed by int
    pub async fn read_dec_float(
        buf: &mut Vec<u8>,
        mut reader: impl AsyncRead + Unpin + AsyncSeek,
    ) -> io::Result<Option<()>> {
        if let None = read_dec_int(buf, &mut reader).await? {
            return Ok(None);
        }
        correct_buf(buf, &mut reader).await?;

        // .
        // return `Some` on eof, since it's not mandatory
        let dot = check_eof!(reader.read_u8().await, eof return Ok(Some(())));
        if dot == b'.' {
            buf.push(dot);

            // [0-9]+
            if let None = read_dec_int(buf, &mut reader).await? {
                return Ok(None);
            }
        }

        correct_buf(buf, &mut reader).await?;

        // e
        // return `Some` on eof, since it's not mandatory
        let e = check_eof!(reader.read_u8().await, eof return Ok(Some(())));
        if e != b'e' {
            println!("no `e`, got {:?}", e as char);
            return Ok(Some(()));
        }
        buf.push(e);

        // [+-]?
        let plus_minus = check_eof!(reader.read_u8().await);
        let was_number = if plus_minus == b'+' || plus_minus == b'-' || plus_minus.is_ascii_digit()
        {
            buf.push(plus_minus);
            plus_minus.is_ascii_digit()
        } else {
            // read `e` but no number after it
            return Ok(None);
        };

        // [0-9]+
        if !was_number {
            if let None = read_dec_int(buf, &mut reader).await? {
                return Ok(None);
            }
        } else {
            // ignore if there was not a number after previous number
            read_dec_int(buf, &mut reader).await?;
        }

        Ok(Some(()))
    }

    // advance past int
    // look for [+-]{0,1}[0-9]+
    pub async fn read_dec_int(
        buf: &mut Vec<u8>,
        mut reader: impl AsyncRead + Unpin,
    ) -> io::Result<Option<()>> {
        let next = check_eof!(reader.read_u8().await);

        // [+-]?
        let mut next = match next {
            b'+' | b'-' => {
                buf.push(next);
                check_eof!(reader.read_u8().await)
            }
            n => n,
        };

        // [0-9]
        if !next.is_ascii_digit() {
            return Ok(None);
        }

        // [0-9]*
        while next.is_ascii_digit() {
            buf.push(next);
            next = check_eof!(
                reader.read_u8().await,
                res => res,
                eof return if buf.len() > 0 {
                    Ok(Some(()))
                } else {
                    Ok(None)
                }
            );
        }

        Ok(Some(()))
    }

    async fn correct_buf(buf: &mut Vec<u8>, mut reader: impl AsyncSeek + Unpin) -> io::Result<()> {
        // read_dec_int (and potentionally others) leaves the reader at the next char,
        // even if it's not parsed correctly, so we seek to the position in the buffer
        // we read so far, so we have consistent behavior
        reader.seek(io::SeekFrom::Start(buf.len() as u64)).await?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use std::io::Cursor;

        use super::*;

        #[tokio::test]
        async fn test_read_dec_int() {
            async fn test(input: &str, expected: Option<i64>) {
                println!("Testing input: {} for {:?}", input, expected);
                let mut reader = Cursor::new(input);
                let mut buf = Vec::new();
                let result = read_dec_int(&mut buf, &mut reader).await.unwrap();
                let string = String::from_utf8(buf).unwrap();
                println!("Resulted string: {}", string);
                assert_eq!(result.map(|_| string.parse().unwrap()), expected);
            }

            test("123", Some(123)).await;
            test("+123", Some(123)).await;
            test("-123", Some(-123)).await;
            // dot is not consumed
            test("123.0", Some(123)).await;
            test("abs", None).await;
            test("123abs", Some(123)).await;
            test("a123", None).await;
            test("", None).await;
        }

        #[tokio::test]
        async fn test_read_dec_float() {
            async fn test(input: &str, expected: Option<f64>) {
                println!("Testing input: {} for {:?}", input, expected);
                let mut reader = Cursor::new(input);
                let mut buf = Vec::new();
                let result = read_dec_float(&mut buf, &mut reader).await.unwrap();
                if result.is_none() {
                    println!("Resulted string: `None`");
                    assert_eq!(None, expected);
                } else {
                    let string = String::from_utf8(buf).unwrap();
                    println!("Resulted string: {}", string);
                    assert_eq!(
                        result.map(|_| string
                            .parse()
                            .expect("read function returned invalid buffer")),
                        expected
                    );
                }
            }

            test("123", Some(123.0)).await;
            test("+123", Some(123.0)).await;
            test("-123", Some(-123.0)).await;
            test("123.5", Some(123.5)).await;
            test("123.e", None).await;
            test("123.0e+1", Some(1230.0)).await;
            test("123.0e-1", Some(12.3)).await;
            test("123.0e1", Some(1230.0)).await;
            test("123.0e", None).await;
            test("123.0e+", None).await;
            test("123.0e-", None).await;
            test("123.0e+abs", None).await;
            test("123.0e-abs", None).await;
            test("123.0eabs", None).await;
            test("123.0e+2", Some(12300.0)).await;
            test("abc", None).await;
            test("123.0e+1e", Some(1230.0)).await;
            test("123.0e-1e", Some(12.3)).await;
        }
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! check_eof {
        ($res:expr, $ok_from:ident => $ok_to:expr, eof $eof:expr) => {{
            match $res {
                Ok($ok_from) => $ok_to,
                Err(err) if matches!(err.kind(), std::io::ErrorKind::UnexpectedEof) => $eof,
                Err(err) => return Err(err.into()),
            }
        }};
        ($res:expr) => {
            check_eof!($res, res => res, eof return Ok(None))
        };
        ($res:expr, eof $eof:expr) => {
            check_eof!($res, res => res, eof $eof)
        };
    }
}
