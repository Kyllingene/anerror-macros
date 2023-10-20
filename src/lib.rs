use proc_macro::TokenStream;

use syn::{parse_macro_input, ItemFn};

/// Sets up nice error handling for your Rust function.
///
/// Technically, this can be applied to any function. However, you only need
/// to apply it to your `main` function. It should never be used in libraries
/// (that applies to the whole crate).
///
/// If you forget this, your pretty errors will all show up as `Box<dyn Any>`.
#[proc_macro_attribute]
pub fn catch(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let sig = input.sig;
    let block = input.block;

    TokenStream::from(quote::quote! {
        #sig {
            let (__errata_tx, __errata_rx) = std::sync::mpsc::channel();

            let __errata_old_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let location = match info.location() {
                    Some(l) => format!(" (at {}:{}:{})", l.file(), l.line(), l.column()),
                    None => String::new(),
                };

                let _ = __errata_tx.send(location);
            }));

            let __errata_result = std::panic::catch_unwind(|| {
                #block
            });

            match __errata_result {
                Ok(_) => return,
                Err(e) => {
                    let location = match __errata_rx.recv() {
                        Ok(l) => l,
                        Err(e) => {
                            eprintln!("internal errata error (failed to recieve location): {e}");
                            String::new()
                        }
                    };

                    if let Some(e) = e.downcast_ref::<errata::ErrataPanic>() {
                        eprintln!("{e}");
                    } else if let Some(e) = e.downcast_ref::<&str>() {
                        eprintln!("error{location}: {e}");

                        let bt = std::backtrace::Backtrace::capture();

                        use std::backtrace::BacktraceStatus as BtS;
                        match bt.status() {
                            BtS::Captured => eprintln!("{bt}"),
                            BtS::Disabled => eprintln!("run with `RUST_BACKTRACE=1` environment variable to display a backtrace"),
                            _ => {}
                        }
                    } else if let Some(e) = e.downcast_ref::<String>() {
                        eprintln!("error{location}: {e}");

                        let bt = std::backtrace::Backtrace::capture();

                        use std::backtrace::BacktraceStatus as BtS;
                        match bt.status() {
                            BtS::Captured => eprintln!("{bt}"),
                            BtS::Disabled => eprintln!("run with `RUST_BACKTRACE=1` environment variable to display a backtrace"),
                            _ => {}
                        }
                    } else {
                        eprintln!("Unhandled error(location)");
                        let bt = std::backtrace::Backtrace::capture();

                        use std::backtrace::BacktraceStatus as BtS;
                        match bt.status() {
                            BtS::Captured => eprintln!("{bt}"),
                            BtS::Disabled => eprintln!("run with `RUST_BACKTRACE=1` environment variable to display a backtrace"),
                            _ => {}
                        }
                    }

                    std::process::exit(1);
                }
            }
        }
    })
}
