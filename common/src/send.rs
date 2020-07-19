use http_body::Body as _;
use hyper::{
    Client,StatusCode,Request,Method,
    client::HttpConnector,
    Body,
};
use hyper_tls::HttpsConnector;
use tokio::{self,runtime::Runtime};

pub struct Intercom {
    client: Client<HttpsConnector<HttpConnector>>,
    server: String,
}

#[derive(Debug)]
pub enum Error {
    Unexpected(hyper::Error),
    UnexpectedStatus(StatusCode),
    Read(hyper::Error),
    Request(String),
    Utf8(std::str::Utf8Error),
}

impl Intercom {
    pub fn proxy() -> Intercom {
        Intercom::new("https://icfpc2020-api.testkontur.ru/aliens/send?apiKey=75f6b337427e482e9308fbe7940031c0".to_string())
    }
    pub fn new(server_url: String) -> Intercom {
        Intercom {
            client: Client::builder().build::<_, hyper::Body>(HttpsConnector::new()),
            server: server_url,
        }
    }
    pub fn send(&self, data: String, runtime: &mut Runtime) -> Result<String,Error> {
        runtime.block_on(self.async_send(data))
    }
    pub async fn async_send(&self, data: String) -> Result<String,Error> {
        let req = Request::builder()
            .method(Method::POST)
            .uri(&self.server)
            .body(Body::from(data)).map_err(|e|Error::Request(format!("{:?}",e)))?;
        match self.client.request(req).await {
            Ok(res) => {
                match res.status() {
                    StatusCode::OK => {
                        let mut tmp = String::new();
                        let mut body = res.into_body();
                        loop {
                            let chunk = match body.data().await {
                                Some(chunk) => chunk,
                                None => break Ok(tmp),
                            };
                            match chunk {
                                Ok(content) => {
                                    //println!("{:?}", content);
                                    tmp += match std::str::from_utf8(&content[..]) {
                                        Err(e) => break Err(Error::Utf8(e)),
                                        Ok(s) => s,
                                    };
                                },
                                Err(e) => break Err(Error::Read(e)),
                            }
                        }
                    },
                    _ => Err(Error::UnexpectedStatus(res.status())),
                }
            },
            Err(e) => Err(Error::Unexpected(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check() {
        let mut runtime = Runtime::new().unwrap();
        let com = Intercom::proxy();
        let r = com.send("1101000".to_string(),&mut runtime).unwrap();
        println!("{}",r);
        assert_eq!(r.starts_with("110110000111"),true)
    }
}

