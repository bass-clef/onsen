#![recursion_limit="512"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod js;
mod model;
mod sen;

/// index.html から呼ばれる最初の init wasm
#[wasm_bindgen(start)]
pub fn run_app() {
    App::<model::MainModel>::new().mount_to_body();
}
