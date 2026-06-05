use serde::Serialize;
use serde_json::json;
use std::io::Read;

const MAX_HEADER_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpRequest {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

pub(crate) fn read_http_request<R: Read>(
    reader: &mut R,
    max_body_bytes: usize,
) -> Result<HttpRequest, HttpResponse> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];

    let header_end = loop {
        let read = reader.read(&mut chunk).map_err(|error| {
            HttpResponse::error(
                400,
                "service.request_read_failed",
                format!("failed to read request: {error}"),
                None,
            )
        })?;
        if read == 0 {
            return Err(HttpResponse::error(
                400,
                "service.empty_request",
                "request was empty".to_string(),
                None,
            ));
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(index) = find_header_end(&buffer) {
            break index;
        }
        if buffer.len() > MAX_HEADER_BYTES {
            return Err(HttpResponse::error(
                400,
                "service.headers_too_large",
                "request headers exceed 16 KiB".to_string(),
                None,
            ));
        }
    };

    let header_text = std::str::from_utf8(&buffer[..header_end]).map_err(|error| {
        HttpResponse::error(
            400,
            "service.invalid_headers",
            format!("request headers are not UTF-8: {error}"),
            None,
        )
    })?;
    let (method, path, content_length) = parse_header_block(header_text)?;
    if content_length > max_body_bytes {
        return Err(HttpResponse::error(
            413,
            "service.body_too_large",
            format!("request body exceeds {max_body_bytes} bytes"),
            None,
        ));
    }

    let body_start = header_end + 4;
    let mut body = buffer[body_start..].to_vec();
    while body.len() < content_length {
        let read = reader.read(&mut chunk).map_err(|error| {
            HttpResponse::error(
                400,
                "service.request_read_failed",
                format!("failed to read request body: {error}"),
                None,
            )
        })?;
        if read == 0 {
            return Err(HttpResponse::error(
                400,
                "service.body_truncated",
                "request body ended before Content-Length bytes were read".to_string(),
                None,
            ));
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(content_length);

    Ok(HttpRequest { method, path, body })
}

fn parse_header_block(header_text: &str) -> Result<(String, String, usize), HttpResponse> {
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let version = parts.next().unwrap_or_default();
    if method.is_empty() || target.is_empty() || !version.starts_with("HTTP/") {
        return Err(HttpResponse::error(
            400,
            "service.invalid_request_line",
            "request line must be '<METHOD> <PATH> HTTP/<VERSION>'".to_string(),
            None,
        ));
    }

    let path = target.split('?').next().unwrap_or(target);
    let mut content_length = 0_usize;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse::<usize>().map_err(|error| {
                    HttpResponse::error(
                        400,
                        "service.invalid_content_length",
                        format!("Content-Length is not a valid integer: {error}"),
                        None,
                    )
                })?;
            }
        }
    }

    Ok((method.to_string(), path.to_string(), content_length))
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

impl HttpRequest {
    #[cfg(test)]
    pub(crate) fn get(path: &str) -> Self {
        Self {
            method: "GET".to_string(),
            path: path.to_string(),
            body: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn post(path: &str, body: Vec<u8>) -> Self {
        Self {
            method: "POST".to_string(),
            path: path.to_string(),
            body,
        }
    }
}

impl HttpResponse {
    pub(crate) fn json<T: Serialize>(status: u16, value: &T) -> Self {
        match serde_json::to_vec_pretty(value) {
            Ok(body) => Self {
                status,
                content_type: "application/json",
                body,
            },
            Err(error) => Self::error(
                500,
                "service.serialization_failed",
                format!("failed to serialize JSON response: {error}"),
                None,
            ),
        }
    }

    pub(crate) fn error(
        status: u16,
        code: &'static str,
        message: String,
        location: Option<String>,
    ) -> Self {
        Self::json(
            status,
            &json!({
                "ok": false,
                "error": {
                    "code": code,
                    "message": message,
                    "location": location
                }
            }),
        )
    }

    pub(crate) fn to_http_bytes(&self) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
            self.status,
            reason_phrase(self.status),
            self.content_type,
            self.body.len()
        )
        .into_bytes();
        response.extend_from_slice(&self.body);
        response
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_get_request() {
        let mut input = b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n".as_slice();
        let request = read_http_request(&mut input, 1024).expect("request");
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/health");
        assert!(request.body.is_empty());
    }

    #[test]
    fn reads_post_body_by_content_length() {
        let mut input =
            b"POST /v0/batch/sequence/validate HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}extra"
                .as_slice();
        let request = read_http_request(&mut input, 1024).expect("request");
        assert_eq!(request.body, b"{}");
    }

    #[test]
    fn rejects_oversize_body_before_reading_payload() {
        let mut input = b"POST /x HTTP/1.1\r\nContent-Length: 3\r\n\r\nabc".as_slice();
        let response = read_http_request(&mut input, 2).expect_err("oversize");
        assert_eq!(response.status, 413);
    }
}
