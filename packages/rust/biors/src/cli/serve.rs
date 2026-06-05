use super::serve_args::ServeArgs;
use super::serve_handlers::handle_service_request;
use super::serve_http::read_http_request;
use crate::errors::CliError;
use std::io::Write;
use std::net::TcpListener;
use std::time::Duration;

pub(crate) fn run_serve(args: ServeArgs) -> Result<(), CliError> {
    let bind_address = args.bind_address();
    let listener = TcpListener::bind(&bind_address).map_err(|error| CliError::Validation {
        code: "service.bind_failed",
        message: format!("failed to bind local service at {bind_address}: {error}"),
        location: Some(bind_address.clone()),
    })?;

    eprintln!(
        "biors serve listening on {} (local-first, no external calls)",
        args.base_url()
    );

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("biors serve accept error: {error}");
                continue;
            }
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let response = match read_http_request(&mut stream, args.max_body_bytes) {
            Ok(request) => {
                handle_service_request(request, env!("CARGO_PKG_VERSION"), &args.base_url())
            }
            Err(response) => response,
        };
        if let Err(error) = stream.write_all(&response.to_http_bytes()) {
            eprintln!("biors serve write error: {error}");
        }
    }

    Ok(())
}
