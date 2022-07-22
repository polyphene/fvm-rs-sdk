//! Error handling for any error that can arise during our procedural macro logic.

use proc_macro2::*;
use quote::{ToTokens, TokenStreamExt};
use syn::parse::Error;

/// Provide a Diagnostic with the given span and message
#[macro_export]
macro_rules! err_span {
    ($span:expr, $($msg:tt)*) => (
        $crate::Diagnostic::spanned_error(&$span, format!($($msg)*))
    )
}

/// Immediately fail and return an Err, with the arguments passed to err_span!
#[macro_export]
macro_rules! bail_span {
    ($($t:tt)*) => (
        return Err(err_span!($($t)*).into())
    )
}

/// A struct representing a diagnostic to emit to the end-user as an error.
#[derive(Debug)]
pub struct Diagnostic {
    inner: Repr,
}

#[derive(Debug)]
enum Repr {
    Single {
        text: String,
        span: Option<(Span, Span)>,
    },
    SynError(Error),
    Multi {
        diagnostics: Vec<Diagnostic>,
    },
}

impl Diagnostic {
    /// Generate a `Diagnostic` from an informational message with no Span
    pub fn error<T: Into<String>>(text: T) -> Diagnostic {
        Diagnostic {
            inner: Repr::Single {
                text: text.into(),
                span: None,
            },
        }
    }

    /// Generate a `Diagnostic` from a Span and an informational message
    pub fn span_error<T: Into<String>>(span: Span, text: T) -> Diagnostic {
        Diagnostic {
            inner: Repr::Single {
                text: text.into(),
                span: Some((span, span)),
            },
        }
    }

    /// Generate a `Diagnostic` from the span of any tokenizable object and a message
    pub fn spanned_error<T: Into<String>>(node: &dyn ToTokens, text: T) -> Diagnostic {
        Diagnostic {
            inner: Repr::Single {
                text: text.into(),
                span: extract_spans(node),
            },
        }
    }

    /// Attempt to generate a `Diagnostic` from a vector of other `Diagnostic` instances.
    /// If the `Vec` is empty, returns `Ok(())`, otherwise returns the new `Diagnostic`
    pub fn from_vec(diagnostics: Vec<Diagnostic>) -> Result<(), Diagnostic> {
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(Diagnostic {
                inner: Repr::Multi { diagnostics },
            })
        }
    }

    /// Immediately trigger a panic from this `Diagnostic`
    #[allow(unconditional_recursion)]
    pub fn panic(&self) -> ! {
        match &self.inner {
            Repr::Single { text, .. } => panic!("{}", text),
            Repr::SynError(error) => panic!("{}", error),
            Repr::Multi { diagnostics } => diagnostics[0].panic(),
        }
    }
}

impl From<Error> for Diagnostic {
    fn from(err: Error) -> Diagnostic {
        Diagnostic {
            inner: Repr::SynError(err),
        }
    }
}

fn extract_spans(node: &dyn ToTokens) -> Option<(Span, Span)> {
    let mut t = TokenStream::new();
    node.to_tokens(&mut t);
    let mut tokens = t.into_iter();
    let start = tokens.next().map(|t| t.span());
    let end = tokens.last().map(|t| t.span());
    start.map(|start| (start, end.unwrap_or(start)))
}

impl ToTokens for Diagnostic {
    fn to_tokens(&self, dst: &mut TokenStream) {
        match &self.inner {
            Repr::Single { text, span } => {
                let cs2 = (Span::call_site(), Span::call_site());
                let (start, end) = span.unwrap_or(cs2);
                dst.append(Ident::new("compile_error", start));
                dst.append(Punct::new('!', Spacing::Alone));
                let mut message = TokenStream::new();
                message.append(Literal::string(text));
                let mut group = Group::new(Delimiter::Brace, message);
                group.set_span(end);
                dst.append(group);
            }
            Repr::Multi { diagnostics } => {
                for diagnostic in diagnostics {
                    diagnostic.to_tokens(dst);
                }
            }
            Repr::SynError(err) => {
                err.to_compile_error().to_tokens(dst);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_type_of<T>(_: &T, expected_type: &str) {
        assert_eq!(expected_type, std::any::type_name::<T>())
    }

    fn assert_span(diagnostic: Diagnostic) {
        match diagnostic.inner {
            Repr::Single { text, span } => {
                assert_eq!(text, String::from("error"));
                match span {
                    Some((start, end)) => {
                        assert_type_of(&start, "proc_macro2::Span");
                        assert_type_of(&end, "proc_macro2::Span");
                    }
                    _ => panic!("span should not be None"),
                }
            }
            _ => panic!("diagnostic inner should be single"),
        }
    }

    #[test]
    fn generate_error_with_message() {
        let diagnostic = Diagnostic::error("error");

        match diagnostic.inner {
            Repr::Single { text, span } => {
                assert_eq!(text, String::from("error"));
                assert!(span.is_none());
            }
            _ => panic!("diagnostic inner should be single"),
        }
    }

    #[test]
    fn generate_error_with_span_message() {
        let mock_ident = Ident::new("mock", Span::call_site());
        let mock_ident_span = mock_ident.span();

        let diagnostic = Diagnostic::span_error(mock_ident_span, "error");

        assert_span(diagnostic);
    }

    #[test]
    fn extract_span() {
        let extracted_span = extract_spans(&"string");
        match extracted_span {
            Some((start, end)) => {
                assert_type_of(&start, "proc_macro2::Span");
                assert_type_of(&end, "proc_macro2::Span");
            }
            None => panic!("extracted span should not be none"),
        }
    }

    #[test]
    fn span_error_from_token() {
        let diagnostic = Diagnostic::spanned_error(&"string", "error");

        assert_span(diagnostic);
    }

    #[test]
    fn error_from_multiple_diagnostic() {
        let diagnostic1 = Diagnostic::error("error1");
        let diagnostic2 = Diagnostic::error("error2");
        let diagnostics_vector = vec![diagnostic1, diagnostic2];

        let ok_res = Diagnostic::from_vec(vec![]);
        assert!(ok_res.is_ok());

        let err_res = Diagnostic::from_vec(diagnostics_vector);
        match err_res {
            Err(diagnostic) => match diagnostic.inner {
                Repr::Multi { diagnostics } => {
                    assert_eq!(diagnostics.len(), 2usize);
                }
                _ => panic!("diagnostic should be of type Multi with non-empty vector"),
            },
            _ => panic!("result should be an error with non-empty vector"),
        }
    }

    #[test]
    fn panic_from_diagnostic() {
        let diagnostic_single = Diagnostic::error("error");
        let diagnostic_multi = Diagnostic::from_vec(vec![
            Diagnostic::error("error1"),
            Diagnostic::error("error2"),
        ])
        .err()
        .unwrap();

        let panic_res_single = std::panic::catch_unwind(|| diagnostic_single.panic());
        match panic_res_single {
            Err(err) => match err.downcast::<String>() {
                Ok(panic_msg_box) => {
                    assert_eq!(panic_msg_box.as_str(), "error");
                }
                Err(_) => unreachable!(),
            },
            _ => panic!("panic() on single diagnostic should be an error in catch_unwind"),
        }

        let panic_res_multi = std::panic::catch_unwind(|| diagnostic_multi.panic());
        match panic_res_multi {
            Err(err) => match err.downcast::<String>() {
                Ok(panic_msg_box) => {
                    assert_eq!(panic_msg_box.as_str(), "error1");
                }
                Err(_) => unreachable!(),
            },
            _ => panic!("panic() on multi diagnostic should be an error in catch_unwind"),
        }
    }

    #[test]
    fn diagnostic_from_syn_error() {
        let mock_ident = Ident::new("mock", Span::call_site());
        let mock_ident_span = mock_ident.span();

        let syn_error = Error::new(mock_ident_span, "error");

        let diagnostic = Diagnostic::from(syn_error);

        match diagnostic.inner {
            Repr::SynError(error) => {
                assert_eq!(error.to_string().as_str(), "error");
            }
            _ => panic!("diagnostic should be of type syn error"),
        }
    }

    #[test]
    fn diagnostic_single_to_tokens() {
        let mut token_stream = TokenStream::new();

        let diagnostic = Diagnostic::error("error");

        diagnostic.to_tokens(&mut token_stream);

        assert_eq!(token_stream.to_string(), "compile_error ! { \"error\" }");
    }

    #[test]
    fn diagnostic_single_span_to_tokens() {
        let mut token_stream = TokenStream::new();

        let mock_ident = Ident::new("mock", Span::call_site());

        let diagnostic = Diagnostic::span_error(mock_ident.span(), "error");

        diagnostic.to_tokens(&mut token_stream);

        assert_eq!(token_stream.to_string(), "compile_error ! { \"error\" }");
    }

    #[test]
    fn diagnostic_multi_to_tokens() {
        let mut token_stream = TokenStream::new();

        let diagnostic = Diagnostic::from_vec(vec![
            Diagnostic::error("error1"),
            Diagnostic::error("error2"),
        ])
        .err()
        .unwrap();

        diagnostic.to_tokens(&mut token_stream);

        assert_eq!(
            token_stream.to_string(),
            "compile_error ! { \"error1\" } compile_error ! { \"error2\" }"
        );
    }

    #[test]
    fn diagnostic_syn_error_to_tokens() {
        let mut token_stream = TokenStream::new();

        let mock_ident = Ident::new("mock", Span::call_site());
        let mock_ident_span = mock_ident.span();

        let syn_error = Error::new(mock_ident_span, "error");

        let diagnostic = Diagnostic::from(syn_error);

        diagnostic.to_tokens(&mut token_stream);

        assert_eq!(token_stream.to_string(), "compile_error ! { \"error\" }");
    }
}
