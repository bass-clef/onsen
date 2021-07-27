use futures::{
    future,
    Future,
};
use js_sys::Promise;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{
    JsFuture,
};
use web_sys::{
    HtmlImageElement,
    Request, RequestInit, RequestMode, Response,
};

#[wasm_bindgen]
extern "C" {
    // 文字の拡大縮小を行ってくれる .js
    pub fn fitty(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = "log")]
    pub fn js_log(s: &str);
}

/// JavaScript 側の console に出力するために移植した
macro_rules! console_log {
    ($($t:tt)*) => (crate::js::js_log(&format_args!($($t)*).to_string()))
}
pub(crate) use console_log;

/// JavaScript の移植、DOMの操作系の関数とか
pub struct DOM;
impl DOM {
    /// 指定の id を DOM から探して返す
    pub fn get_element_by_id<T: JsCast>(id: &str) -> T {
        let window = web_sys::window().expect("no global `window` exist");
        let document = window.document().expect("should have a document on window");

        document
            .get_element_by_id(id)
            .expect(&format!("not found element id {}", id))
            .dyn_into::<T>()
            .map_err(|_| ())
            .unwrap()
    }

    /// src がついた img タグを返す
    /// alt は自動でファイル名にする
    pub fn make_img_element<'a>(path: &'a str) -> HtmlImageElement {
        let image = HtmlImageElement::new().unwrap();
        image.set_src(path);
        image.set_alt( std::path::Path::new(path).file_stem().unwrap().to_str().unwrap() );

        image
    }
}

pub mod fs {
    use super::*;
    
    #[wasm_bindgen(inline_js = "export function get_sync_request(url) { let xhr = new XMLHttpRequest(); xhr.open('GET', url, false); xhr.responseType = 'text/plain'; xhr.setRequestHeader( 'X-Requested-With', 'XMLHttpRequest' ); xhr.send(); if (xhr.status == 200){ return xhr.responseText;} }")]
    extern "C" {
        pub fn get_sync_request(path: &str) -> String;
    }
    
    /// FileIO の wasm 版
    /// XMLHttpRequest で get リクエストして file を同期でもらってる
    /// 下記のと同義
    /// <pre>
    /// let file = std::fs::File::open( path: &str ).unwrap();
    /// </pre>
    pub fn open<'a>(file_path: String) -> stringreader::StringReader<'a> {
        let data = get_sync_request(&file_path);

        // move を許可してくれない参照系男子 (おそらくプログラム終了時に開放されるので問題ない?)
        stringreader::StringReader::new( Box::leak(data.into_boxed_str()) )
    }
}
