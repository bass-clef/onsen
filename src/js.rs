use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;

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
