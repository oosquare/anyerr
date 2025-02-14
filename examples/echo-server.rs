mod err {
    use anyerr::context::LiteralKeyStringMapContext;
    use anyerr::AnyError as AnyErrorTemplate;

    pub use anyerr::kind::NoErrorKind as ErrKind;
    pub use anyerr::Report;
    pub use anyerr::{Intermediate, Overlay};

    pub type AnyError = AnyErrorTemplate<LiteralKeyStringMapContext, ErrKind>;
    pub type AnyResult<T> = Result<T, AnyError>;
}

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Termination;
use std::thread;
use std::time::Duration;

use err::*;

const SERVER_IP: &str = "127.0.0.1";
const SERVER_PORT: &str = "8080";

fn main() -> impl Termination {
    Report::capture(|| {
        let listener = TcpListener::bind(format!("{SERVER_IP}:{SERVER_PORT}"))
            .map_err(AnyError::wrap)
            .overlay("could not bind the listener to the endpoint")
            .context("ip", SERVER_IP)
            .context("port", SERVER_PORT)?;

        eprintln!("Started listening on {SERVER_IP}:{SERVER_PORT}");

        for connection in listener.incoming() {
            let Ok(stream) = connection else {
                continue;
            };

            thread::spawn(move || {
                handle_connection(stream).unwrap_or_else(|err| {
                    let report = Report::wrap(err).kind(false);
                    eprintln!("{report}");
                });
            });
        }

        Ok(())
    })
    .kind(false)
}

fn handle_connection(mut stream: TcpStream) -> AnyResult<()> {
    let client_addr = stream
        .peer_addr()
        .map_or("<UNKNOWN>".into(), |addr| addr.to_string());
    let mut buffer = [0u8; 256];
    let mut total_read = 0;

    eprintln!("{client_addr} started the connection");
    thread::sleep(Duration::from_secs(3));

    loop {
        let size_read = stream
            .read(&mut buffer)
            .map_err(AnyError::wrap)
            .overlay("could not read bytes from the client")
            .context("client_addr", &client_addr)
            .context("total_read", total_read)?;
        total_read += size_read;

        if size_read == 0 {
            eprintln!("{client_addr} closed the connection");
            return Ok(());
        }

        thread::sleep(Duration::from_secs(3));

        let mut cursor = 0;
        while cursor < size_read {
            let size_written = stream
                .write(&buffer[cursor..size_read])
                .map_err(AnyError::wrap)
                .overlay("could not write bytes to the client")
                .context("client_addr", &client_addr)
                .context("total_read", total_read)
                .context("cursor", cursor)?;
            cursor += size_written;
        }
    }
}
