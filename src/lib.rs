#![recursion_limit="512"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod js;
mod model;
mod sen;

/// index.html から呼ばれる最初の init wasm
#[wasm_bindgen(start)]
pub fn run_app() {
    set_panic_hook();

    App::<model::MainModel>::new().mount_to_body();
}

fn set_panic_hook() {
    // パニック時のフックを登録する
    // https://qiita.com/mizkichan/items/da2d73ed3dc6c2b43b1d
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();

        // payload は Any なので String か &str だったら具象型に変換し Some にくるむ
        let payload = if payload.is::<String>() {
            Some(payload.downcast_ref::<String>().unwrap().as_str())
        } else if payload.is::<&str>() {
            Some(*payload.downcast_ref::<&str>().unwrap())
        } else {
            None
        };

        // PC での panic 表示を真似てフォーマット
        if let (Some(payload), Some(location)) = (payload, panic_info.location()) {
            js::console_log!("panicked at {:?}, {}:{}:{}", payload, location.file(), location.line(), location.column());
        }
    }));
}