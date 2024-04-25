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
    //! You can also notice that some functiond have a `PhantomData<T>` in a return
    //! type. This is to ensure that the type `T` is not actually constructed.
    //! But we can encode the type `T` in the return type, so that the caller
    //! can use it to parse the type `T` if needed. Pretty neat, huh?

    use std::{fmt::Debug, future::Future, marker::PhantomData, str::FromStr};

    use tokio::io::{self, AsyncRead, AsyncReadExt};

    use crate::check_eof;

    pub(crate) async fn parsed<R, P, F>(
        func: fn(&mut Vec<u8>, R) -> F,
        reader: R,
    ) -> io::Result<Option<P>>
    where
        F: Future<Output = io::Result<Option<PhantomData<P>>>>,
        P: FromStr,
        <P as FromStr>::Err: Debug,
    {
        let mut buf = Vec::new();
        let res = func(&mut buf, reader).await;

        // should be safe to unwrap here,
        // provided the function `func` is implemented correctly.
        // That's why it's pub(crate).
        let string = String::from_utf8(buf).unwrap();

        match res {
            // also should be safe to expect correct format
            Ok(Some(_)) => Ok(Some(string.parse().unwrap())),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    // advance past float
    // look for [+-]{0,1}[0-9]+ // int
    // then optional .[0-9]+
    // then optional e[+-]{0,1}[0-9]+
    pub async fn read_float(
        buf: &mut Vec<u8>,
        reader: impl AsyncRead + Unpin,
    ) -> io::Result<Option<PhantomData<f64>>> {
        // let next = reader.read_u8().await?;

        todo!()
    }

    // advance past int
    // look for [+-]{0,1}[0-9]+
    pub async fn read_dec_int<R>(
        buf: &mut Vec<u8>,
        mut reader: R,
    ) -> io::Result<Option<PhantomData<i64>>>
    where
        R: AsyncRead + Unpin,
    {
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
                res,
                res,
                return if buf.len() > 0 {
                    Ok(Some(PhantomData))
                } else {
                    Ok(None)
                }
            );
        }

        Ok(Some(PhantomData))
    }

    #[cfg(test)]
    mod tests {
        use std::io::Cursor;

        use super::*;

        #[tokio::test]
        async fn test_read_dec_int() {
            async fn test(input: &str, expected: Option<i64>) {
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
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! check_eof {
        ($res:expr, $ok_from:ident, $ok_to:expr, $eof:expr) => {{
            match $res {
                Ok($ok_from) => $ok_to,
                Err(err) => {
                    if matches!(err.kind(), std::io::ErrorKind::UnexpectedEof) {
                        $eof
                    } else {
                        return Err(err.into());
                    }
                }
            }
        }};
        ($res:expr) => {
            check_eof!($res, res, res, return Ok(None))
        };
    }
}
