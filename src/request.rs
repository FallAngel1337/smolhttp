use super::HttpError;
use super::Response;
use super::TlsConnector;
use super::Url;

use std::{io::prelude::*, str::FromStr, fmt::Display};
use std::net::TcpStream;
use std::time;

#[derive(Debug, Clone)]
pub enum Methods {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    OPTIONS,
    CUSTOM(String),
}

///proxy info object.
#[derive(Debug, Clone)]
pub struct Proxy {
    host: String,
    port: u16,
    scheme: String,
    url: Url,
}

///http request object.
#[derive(Debug, Clone)]
pub struct Client {
    host: String,
    port: u16,
    scheme: String,
    method: Methods,
    url: Url,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    timeout: u64,
    proxy: Option<Proxy>,
    verify: bool,
}

impl Display for Methods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::Methods::*;
        match self {
            GET => write!(f, "GET"),
            POST => write!(f, "POST"),
            PUT => write!(f, "PUT"),
            HEAD => write!(f, "HEAD"),
            DELETE => write!(f, "DELETE"),
            OPTIONS => write!(f, "OPTIONS"),
            CUSTOM(method) => write!(f, "{method}"),
        }
    }
}

impl Client {
    ///return a Request object
    /// # Example
    /// ```
    /// use minihttp::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// ```
    pub fn new(url: &str) -> Result<Self, HttpError> {
        let url: Url = Url::parse(url);

        let host = match url.host {
            Some(ref h) => h.clone(),
            None => return Err(HttpError::Parse("url parse error")),
        };
        Ok(Self {
            host,
            port: url.port,
            scheme: url.scheme.clone(),
            method: Methods::GET,
            url,
            headers: Vec::new(),
            body: None,
            timeout: 30,
            proxy: None,
            verify: true,
        })
    }

    ///set Request GET method
    /// # Example
    /// ```
    /// use minihttp::CLient;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.get();
    /// ```
    pub fn get(&mut self) -> &mut Self {
        self.method = Methods::GET;
        self
    }

    ///set Request POST method
    /// # Example
    /// ```
    /// use minihttp::CLient;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.post();
    /// ```
    pub fn post(&mut self) -> &mut Self {
        self.method = Methods::POST;
        self
    }

    ///set Request PUT method
    /// # Example
    /// ```
    /// use minihttp::CLient;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.put();
    /// ```
    pub fn put(&mut self) -> &mut Self {
        self.method = Methods::PUT;
        self
    }

    ///set Request HEAD method
    /// # Example
    /// ```
    /// use minihttp::CLient;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.head();
    /// ```
    pub fn head(&mut self) -> &mut Self {
        self.method = Methods::HEAD;
        self
    }

    ///set Request DELETE method
    /// # Example
    /// ```
    /// use minihttp::CLient;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.delete();
    /// ```
    pub fn delete(&mut self) -> &mut Self {
        self.method = Methods::DELETE;
        self
    }

    ///set Request OPTION method
    /// # Example
    /// ```
    /// use minihttp::CLient;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.options();
    /// ```
    pub fn options(&mut self) -> &mut Self {
        self.method = Methods::OPTIONS;
        self
    }

    ///set Client's custom method
    /// # Example
    /// ```
    /// use minihttp::Client;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.request("profile");
    /// ```
    pub fn request(&mut self, method: &str) -> &mut Self {
        self.method = Methods::CUSTOM(method.to_owned());
        self
    }

    ///set Client's custom header
    /// # Example
    /// ```
    /// use minihttp::Client;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// let mut headers = vec![("User-Agent".to_owned(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_owned())]
    /// client.headers(headers);
    /// ```
    pub fn headers(&mut self, data: Vec<(String, String)>) -> &mut Self {
        self.headers = data;
        self
    }

    ///set Client's body
    /// # Example
    /// ```
    /// use minihttp::Client;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// let body = vec![0,1,2,3,4];
    /// client.body(body);
    /// ```
    pub fn body(&mut self, data: Vec<u8>) -> &mut Self {
        self.body = Some(data);
        self
    }

    ///set Client's read/write timeout(sec)
    /// # Example
    /// ```
    /// use minihttp::Client;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.timeout(10);
    /// ```
    pub fn timeout(&mut self, time: u64) -> &mut Self {
        self.timeout = time;
        self
    }

    ///set http(s) request if verify the certificate(default true)
    /// # Example
    /// ```
    /// use minihttp::Client;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.verify(false);
    /// ```
    pub fn verify(&mut self, verify: bool) -> Result<&mut Self, HttpError> {
        if self.scheme == "https" {
            self.verify = verify;
        } else {
            return Err(HttpError::Config("Verify setting only for https"));
        }
        Ok(self)
    }

    ///set proxy info
    /// # Example
    /// ```
    /// use minihttp::Client;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.proxy("https://127.0.0.1:1080");
    /// ```
    pub fn proxy(&mut self, proxy: &str) -> Result<&mut Self, HttpError> {
        let url: Url = Url::parse(proxy);
        if self.scheme == "https" && url.scheme == "http" {
            return Err(HttpError::Proxy("Http proxy can only use http scheme."));
        }

        let host = match url.host {
            Some(ref h) => h.clone(),
            None => return Err(HttpError::Parse("url parse error")),
        };

        let proxy = Proxy {
            host,
            port: url.port,
            scheme: url.scheme.clone(),
            url,
        };
        self.proxy = Some(proxy);
        Ok(self)
    }

    ///send http(s) request
    /// # Example
    /// ```
    /// use minihttp::Client;
    ///
    /// let mut client = Client::new("https://www.google.com").unwrap();
    /// client.request("GET").send();
    /// ```
    pub fn send(&mut self) -> Result<Response, HttpError> {
        if let Some(ref proxy) = self.proxy {
            if proxy.scheme == "http" {
                let header = self.build_header();
                let addr = format!("{}:{}", proxy.host, proxy.port);
                let mut stream = TcpStream::connect(addr)?;
                stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.write_all(header.as_bytes())?;
                if let Some(ref body) = self.body {
                    stream.write_all(body.as_slice())?;
                }
                stream.flush()?;
                let mut res: Vec<u8> = Vec::new();
                stream.read_to_end(&mut res)?;
                let back = Response::new(res)?;
                Ok(back)
            } else {
                //CONNECT proxy.google.com:443 HTTP/1.1
                //Host: www.google.com:443
                //Proxy-Connection: keep-alive
                let mut connect_header = String::new();
                connect_header
                    .push_str(&format!("CONNECT {}:{} HTTP/1.1\r\n", self.host, self.port));
                connect_header.push_str(&format!("Host: {}:{}\r\n", self.host, self.port));
                connect_header.push_str("\r\n");
                let addr = format!("{}:{}", proxy.host, proxy.port);
                let mut stream = TcpStream::connect(addr)?;
                stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.write_all(connect_header.as_bytes())?;
                stream.flush()?;

                //HTTP/1.1 200 Connection Established
                let mut res = [0u8; 1024];
                stream.read_exact(&mut res)?;

                let res_s = match String::from_utf8(res.to_owned().to_vec()) {
                    Ok(r) => r,
                    Err(_) => return Err(HttpError::Proxy("parse proxy server response error.")),
                };
                if !res_s
                    .to_ascii_lowercase()
                    .contains("connection established")
                {
                    return Err(HttpError::Proxy("Proxy server response error."));
                }

                if self.scheme == "http" {
                    let header = self.build_header();
                    stream.write_all(header.as_bytes())?;
                    if let Some(ref body) = self.body {
                        stream.write_all(body.as_slice())?;
                    }
                    stream.flush()?;
                    let mut res: Vec<u8> = Vec::new();
                    stream.read_to_end(&mut res)?;
                    let back = Response::new(res)?;
                    Ok(back)
                } else {
                    let connector = TlsConnector::builder().build()?;
                    let mut ssl_stream;
                    ssl_stream = connector.connect(&&self.host, stream)?;
                    let header = self.build_header();
                    ssl_stream.write_all(header.as_bytes())?;
                    if let Some(ref body) = self.body {
                        ssl_stream.write_all(body.as_slice())?;
                    }
                    ssl_stream.flush()?;
                    let mut res: Vec<u8> = Vec::new();
                    ssl_stream.read_to_end(&mut res)?;
                    let back = Response::new(res)?;
                    Ok(back)
                }
            }
        } else if self.scheme == "http" {
            let header = self.build_header();
            let addr = format!("{}:{}", self.host, self.port);
            let mut stream = TcpStream::connect(addr)?;
            stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
            stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
            stream.write_all(header.as_bytes())?;
            if let Some(ref body) = self.body {
                stream.write_all(body.as_slice())?;
            }
            stream.flush()?;
            let mut res: Vec<u8> = Vec::new();
            stream.read_to_end(&mut res)?;
            let back = Response::new(res)?;
            Ok(back)
        } else {
            let addr = format!("{}:{}", self.host, self.port);
            let stream = TcpStream::connect(addr)?;
            stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
            stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
            let connector = TlsConnector::builder().build()?;
            let mut ssl_stream;

            ssl_stream = connector.connect(&&self.host, stream)?;

            let header = self.build_header();
            ssl_stream.write_all(header.as_bytes())?;
            if let Some(ref body) = self.body {
                ssl_stream.write_all(body.as_slice())?;
            }
            ssl_stream.flush()?;

            let mut res: Vec<u8> = Vec::new();
            ssl_stream.read_to_end(&mut res)?;
            let back = Response::new(res)?;
            Ok(back)
        }
    }

    //build http request headers
    fn build_header(&self) -> String {
        if let Some(ref proxy) = self.proxy {
            if proxy.scheme == "http" {
                let mut headers = String::new();
                headers.push_str(&format!(
                    "{} {} HTTP/1.1\r\n",
                    self.method,
                    self.url.as_string()
                ));
                headers.push_str(&format!("Host: {}:{}\r\n", self.host, self.port));
                headers.push_str("Connection: Close\r\n");
                if let Some(ref body) = self.body {
                    headers.push_str(&format!("Content-Length: {}\r\n", body.len()));
                }
                for (i, k) in &self.headers {
                    headers.push_str(&format!("{}: {}\r\n", i, k));
                }
                headers.push_str("\r\n");
                headers
            } else {
                let mut headers = String::new();
                headers.push_str(&format!(
                    "{} {} HTTP/1.1\r\n",
                    self.method,
                    self.url.request_string()
                ));
                headers.push_str(&format!("Host: {}:{}\r\n", self.host, self.port));
                headers.push_str("Connection: Close\r\n");
                if let Some(ref body) = self.body {
                    headers.push_str(&format!("Content-Length: {}\r\n", body.len()));
                }
                for (i, k) in &self.headers {
                    headers.push_str(&format!("{}: {}\r\n", i, k));
                }
                headers.push_str("\r\n");
                headers
            }
        } else {
            let mut headers = String::new();
            headers.push_str(&format!(
                "{} {} HTTP/1.1\r\n",
                self.method,
                self.url.request_string()
            ));
            headers.push_str(&format!("Host: {}:{}\r\n", self.host, self.port));
            headers.push_str("Connection: Close\r\n");
            if let Some(ref body) = self.body {
                headers.push_str(&format!("Content-Length: {}\r\n", body.len()));
            }
            for (i, k) in &self.headers {
                headers.push_str(&format!("{}: {}\r\n", i, k));
            }
            headers.push_str("\r\n");
            headers
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_get() {
        let mut http = Client::new("https://docs.rs/").unwrap();
        println!(
            "{}",
            http.verify(false)
                .unwrap()
                .get()
                .send()
                .unwrap()
                .status_code()
        )
    }

    #[test]
    fn http_post() {
        let mut http = Client::new("https://docs.rs/").unwrap();
        println!(
            "{}",
            http
                .body("username=bob".as_bytes().to_vec())
                .post()
                .send()
                .unwrap()
                .status_code()
        )
    }

    #[test]
    fn http_get_set_header() {
        let mut http = Client::new("https://docs.rs/").unwrap();
        println!(
            "{}",
            http.headers(vec![("Content-Type".to_string(), "text/html; charset=utf-8".to_string())]).get().send().unwrap().status_code()
        )
    }

    #[test]
    fn http_get_back_header() {
        let mut http = Client::new("https://docs.rs/").unwrap();
        let headers = http.get().send().unwrap().headers();
        for (k, v) in headers {
            println!("{}:{}", k, v);
        }
    }

    #[test]
    fn http_proxy() {
        let mut http = Client::new("https://docs.rs/").unwrap();
        let res = http
            .proxy("https://127.0.0.1:1080")
            .unwrap()
            .get()
            .send()
            .unwrap();
        println!("{}", res.status_code());
    }

}
