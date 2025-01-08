#![no_std]
#![no_main]

extern crate alloc;
use alloc::{format, rc::Rc, string::{String, ToString}};
use core::cell::RefCell;
use net_wasabi::http::HttpClient;
use noli::*;
use saba_core::{browser::Browser, error::Error, http::HttpResponse, url::Url};
use ui_wasabi::app::WasabiUI;


fn handle_url(url: String) -> Result<HttpResponse, Error> {
    // URLを解釈する
    let parsed_url = match Url::new(url.to_string()).parse() {
        Ok(url) => url,
        Err(e) => {
            return Err(Error::UnexpectedInput(format!(
                "input html is not supported: {:?}",
                e
            )));
        }
    };

    // HTTPクライアントを作成
    let client = HttpClient::new();
    // HTTPリクエストを送信する
    let response = match client.get(
        parsed_url.host(),
        parsed_url.port().parse::<u16>().expect(&format!(
            "port number should be u16 but got {}",
            parsed_url.port()
        )),
        parsed_url.path()
    ) {
        // レスポンスが無事に取得できた場合
        Ok(res) => {
            // HTTPレスポンスのステータスコードが302のときは転送する
            if res.status_code() == 302 {
                // 転送先を示すLocationヘッダーの値を取得
                let location = match res.header_value("Location") {
                    Ok(value) => value,
                    Err(_) => return Ok(res)
                };

                // 転送先のURLを作成
                let redirect_parsed_url = Url::new(location);

                // 転送のリクエストを実行する
                let redirect_res = match client.get(
                    redirect_parsed_url.host(),
                    redirect_parsed_url.port().parse::<u16>().expect(&format!(
                        "port number should be u16 but got {}",
                        parsed_url.port()
                    )),
                    redirect_parsed_url.path()
                ) {
                    Ok(res) => res,
                    Err(e) => return Err(Error::Network(format!("{:?}", e)))
                };

                redirect_res
            }  else {
                res
            }
        }
        Err(e) => {
            return Err(Error::Network(format!(
                "failed to get http response: {:?}", 
                e
            )))
        }

    };

    Ok(response)
    
}


fn main() -> u64 {
    // Browser構造体を初期化
    let browser = Browser::new();
    
    // WasabiUI構造体を初期化
    let ui = Rc::new(RefCell::new(WasabiUI::new(browser)));

    // アプリの実行を開始
    match ui.borrow_mut().start(handle_url) {
        Ok(_) => {},
        Err(e) => {
            println!("browser fails to start: {:?}", e);
            return 1;
        }
    }

    0
}

entry_point!(main);

