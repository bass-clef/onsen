use futures::{
    future,
    Future,
};
use js_sys::Promise;
use wasm_bindgen::{
    JsCast,
    JsValue,
};
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

#[wasm_bindgen(inline_js = "export function delete_all_cache() { caches.keys().then(function(keyList) { return Promise.all(keyList.map(function(key) { caches.delete(key); })); }); }")]
extern "C" {
    /// キャッシュの削除
    pub fn delete_all_cache();
}

/// JavaScript 側の console に出力するために移植した
macro_rules! console_log {
    ($($t:tt)*) => (crate::js::js_log(&format_args!($($t)*).to_string()))
}
pub(crate) use console_log;

/// JavaScript の移植、DOMの操作系の関数とか
pub mod dom {
    use super::*;

    /// 指定の id を DOM から探して返す
    pub fn get_element_by_id<T: JsCast>(id: &str) -> Option<T> {
        let window = web_sys::window().expect("no global `window` exist");
        let document = window.document().expect("should have a document on window");

        match document.get_element_by_id(id) {
            Some(element) => {
                Some(
                    element.dyn_into::<T>()
                        .map_err(|_| ())
                        .unwrap()
                )
            },
            None => None,
        }
    }

    /// src がついた img タグを返す
    /// alt は自動でファイル名にする
    pub fn make_img_element<'a>(src: &'a str) -> HtmlImageElement {
        let image = HtmlImageElement::new().unwrap();
        image.set_src(src);
        image.set_alt( std::path::Path::new(src).file_stem().unwrap().to_str().unwrap() );

        image
    }

    /// redirect をする (location.replace(url) )
    pub fn redirect(url: &str) {
        let window = web_sys::window().expect("no global `window` exist");

        window.location().replace(url).unwrap();
    }

    /// リダイレクト ～キャッシュバスターとともに～
    pub fn super_redirect(url: &str) {
        redirect(&format!( "{}?cache_buster={:?}", url, time::now().as_f64().unwrap() ));
    }
}

pub mod fs {
    use super::*;
    
    #[wasm_bindgen(inline_js = "export function get_sync_request(url) { let xhr = new XMLHttpRequest(); xhr.open('GET', url, false); xhr.responseType = 'text/plain'; xhr.setRequestHeader( 'X-Requested-With', 'XMLHttpRequest' ); xhr.send(); if (xhr.status == 200){ return xhr.responseText;} }")]
    extern "C" {
        /// GETリクエストを同期で path に対して行う
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

        // WARNING: str を static として保持しておくけど、おそらくプログラム終了時に開放されるので問題ない?
        stringreader::StringReader::new( Box::leak(data.into_boxed_str()) )
    }

    /// データを LocalStorage でやり取りするための getter (ファイルの代わり)
    pub fn read_storage<'a>(name: String) -> Option<stringreader::StringReader<'a>> {
        let window = web_sys::window().expect("no global `window` exist");
        let storage = match window.local_storage().unwrap() {
            Some(storage) => storage,
            None => return None,
        };

        let data = storage.get_item(&name).unwrap();

        // WARNING: str を static として保持しておくけど、おそらくプログラム終了時に開放されるので問題ない?
        match data {
            Some(data) => Some(stringreader::StringReader::new( Box::leak(data.into_boxed_str()) )),
            None => None,
        }
    }

    /// データを LocalStorage でやり取りするための setter (ファイルの代わり)
    pub fn write_storage(name: String, data: String) {
        let window = web_sys::window().expect("no global `window` exist");
        let storage = window.local_storage().unwrap().expect("no local storage exist. don't write.");
        storage.set_item(&name, &data).unwrap();
    }
}

pub mod time {
    use super::*;

    #[wasm_bindgen(inline_js = "export function now(url) { return Date.now(); }")]
    extern "C" {
        /// どうやら Rust::std::time::Instant::now は未実装っぽいので JavaScript からもらう
        pub fn now() -> JsValue;
    }
}
