use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use yew::prelude::{
    html,
    Component,
    ComponentLink,
    Html,
    ShouldRender,
};
use web_sys::*;

use crate::sen;
use crate::js;

// 問題
#[derive(Debug, Default, Deserialize, Serialize)]
struct OnsenStatus {
    is_clear: bool,
    temperature: i32,
    sen: sen::SenManager,
}

// 問題を管理するクラス
#[derive(Debug, Deserialize, Serialize)]
struct QuastionManager {
    pub quastion_list: HashMap<String, OnsenStatus>,
}
impl QuastionManager {
    const FILE_PATH: &'static str = "/data/quastions.json";

    fn new() -> Self {
        let file = js::fs::open( Self::FILE_PATH.to_string() );
        let reader = std::io::BufReader::new(file);

        match serde_json::from_reader::<_, Self>(reader) {
            Ok(quastion_list) => {
                quastion_list
            },
            Err(e) => {
                panic!("invalid quastions.json E:{:?}", e);
            }
        }
    }
}

/// 一度だけ読み込む必要があるリソースを保持するクラス
struct Resource {
    quastion_manager: Option<QuastionManager>,
}
impl Resource {
    fn get_quastion_manager (&mut self) -> &QuastionManager {
        if self.quastion_manager.is_none() {
            self.quastion_manager = Some(QuastionManager::new());
        }

        self.quastion_manager.as_ref().unwrap()
    }
}
static mut RESOURCE: Resource = Resource {
    quastion_manager: None,
};

/// イベントメッセージ
pub enum Message {
    TouchStart(web_sys::TouchEvent),
    TouchMove(web_sys::TouchEvent),
    TouchEnd,
    TouchStartBackSen,
    TouchStartFrontSen,
}
  
/// ページ遷移用の雛形
trait PageTrait {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html;

    fn update(&mut self, _message: Message) {}
    fn rendered(&mut self, _first_render: bool) {}
}

// 問題出題ページは Topページ も兼ねている
struct QuastionPage {
    sen_op: sen::SenOpManager,
    quastion: &'static OnsenStatus,
    now_status: OnsenStatus,
    cursor_image: Option<HtmlImageElement>,
}
impl QuastionPage {
    fn new() -> Self {
        let mut own = Self {
            sen_op: sen::SenOpManager::new( vec![sen::SenOp::Off, sen::SenOp::On, sen::SenOp::Or, sen::SenOp::And, sen::SenOp::Not], Some(1) ),
            quastion: &unsafe{RESOURCE.get_quastion_manager()}.quastion_list["top"],
            now_status: OnsenStatus::default(),
            cursor_image: None,
        };
        own.load_quastion("top");

        own
    }

    // 問題の読み込みと現在の状態の初期化
    fn load_quastion(&mut self, name: &str) {
        self.quastion = &unsafe{RESOURCE.get_quastion_manager()}.quastion_list[ name ];
        self.now_status.sen.deep_copy(&self.quastion.sen);
    }
}
impl PageTrait for QuastionPage {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        html! {
            <div class="container" id="grand_parent_node">
                <div class="container_item_header">
                    /*{ "banner_area" }*/
                </div>
                <div class="container_item_title">
                    <div id="title_text">
                        <ruby>
                            <div
                                data-back-ruby={ self.sen_op.get_back() }
                                ontouchstart=link.callback(|_| Message::TouchStartBackSen)
                            ></div>
                            <rb ontouchstart=link.callback(|_| Message::TouchStartFrontSen)>
                                { self.sen_op.get_top() }
                            </rb>
                            <div
                                data-front-ruby={ self.sen_op.get_front() }
                                ontouchstart=link.callback(|_| Message::TouchStartFrontSen)
                            ></div>
                        </ruby>
                        { "Sen" }
                        <img id="onsen_mark" src={ self.sen_op.get_top().to_file_name() } alt="onsen_mark"
                            ontouchstart=link.callback(|event| Message::TouchStart(event))
                            ontouchmove=link.callback(|event| Message::TouchMove(event))
                            ontouchend=link.callback(|_| Message::TouchEnd)
                            />
                    </div>
                </div>
                <div class="container_item_content">
                    <img id="open_air_bath" src="/resource/onsen_ofuro_noone_bg.png" alt="open_air_bath" />
                    <div id="wood_kanban_div">
                        <img id="wood_kanban" src="/resource/wood_kanban.png" alt="wood_kanban" />
                        <div id="wood_kanban_text_div">
                            <div id="wood_kanban_text">{ self.quastion.temperature }{ "℃" }</div>
                        </div>
                    </div>
                    <div id="ondokei_div">
                        <img id="ondokei" src="/resource/ondokei.png" alt="ondokei" />
                        <div id="ondokei_text_div">
                            <p id="ondokei_text">{ self.now_status.temperature }{ "℃" }</p>
                        </div>
                    </div>
                    <div id="onsen_mark_frame_div">
                        <img id="onsen_mark_frame" src="/resource/onsen_mark_frame.png" alt="onsen_mark_frame" />
                        <div id="onsen_mark_sen">
                        </div>
                    </div>
                </div>
                <div class="container_item_footer">
                    /*{ "banner_area" }*/
                </div>
            </div>
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::TouchStart(event) => {
                // ドラッグ中の温泉マークを作成
                let x = event.touches().get(0).unwrap().client_x();
                let y = event.touches().get(0).unwrap().client_y();
                let parent_image: HtmlImageElement = js::DOM::get_element_by_id("onsen_mark");
                
                let image: HtmlImageElement = js::DOM::make_img_element( &parent_image.current_src() );
                image.set_id("temp_onsen_mark");
                image.style().set_property("left", &format!("{}px", x) ).unwrap();
                image.style().set_property("top", &format!("{}px", y) ).unwrap();
                self.cursor_image = Some(image);

                js::DOM::get_element_by_id::<HtmlElement>("grand_parent_node")
                    .append_child(self.cursor_image.as_ref().unwrap()).unwrap();
            },
            Message::TouchMove(event) => {
                // ドラッグ中の温泉マークの座標をカーソルに追尾させる
                let x = event.touches().get(0).unwrap().client_x();
                let y = event.touches().get(0).unwrap().client_y();
                if let Some(image) = &self.cursor_image {
                    image.style().set_property("left", &format!("{}px", x) ).unwrap();
                    image.style().set_property("top", &format!("{}px", y) ).unwrap();
                }
            },
            Message::TouchEnd => {
                // ドロップ時に sen と重なっているかを判断して、重なってるなら演算をする
                let mut x: f64 = 0.0;
                let mut y: f64 = 0.0;
                if let Some(image) = &self.cursor_image {
                    let left: String = image.style().get_property_value("left").unwrap();
                    x = left.split_at( left.len() - 2 ).0.parse::<f64>().unwrap();
                    let top: String = image.style().get_property_value("top").unwrap();
                    y = top.split_at( top.len() - 2 ).0.parse::<f64>().unwrap();
                }

                let sen_op_top = self.sen_op.get_top();
                let mut new_op = self.now_status.sen.operation(&self.sen_op.get_top(), &mut |(index, _sen)| {
                    let sen_image: HtmlImageElement = js::DOM::get_element_by_id(&format!("bit_{}", index));
                    let rect = sen_image.get_bounding_client_rect();

                    // 短角内 は各項目に適用, On, Off は全てに適用
                    rect.x() <= x && x < rect.x()+rect.width() && rect.y() <= y && y < rect.y()+rect.height()
                        || sen_op_top == sen::SenOp::On
                        || sen_op_top == sen::SenOp::Off
                });

                if self.sen_op.get_top().is_o() {
                    // もし SenOp::O なら消費する
                    self.sen_op.pop();
                }

                self.sen_op.append_at_index(&mut new_op);
                js::console_log!("{:?} / {:?}", self.sen_op, self.now_status.sen);
                self.now_status.temperature = self.now_status.sen.get_number();

                // ドラッグ中の温泉マークを削除
                if let Some(image) = self.cursor_image.as_mut() {
                   image.remove();
                   self.cursor_image = None;
                }
            },
            Message::TouchStartBackSen => self.sen_op.prev(),
            Message::TouchStartFrontSen => self.sen_op.next(),
            _ => (),
        }
    }

    fn rendered(&mut self, first_render: bool) {
        // onsen_mark_sen に動的にぶら下げるので sen はここで表示
        let onsen_mark_sen: HtmlElement = js::DOM::get_element_by_id("onsen_mark_sen");
        self.now_status.sen.for_each(|(index, sen)|{
            if first_render {
                // 作成
                let sen_image = js::DOM::make_img_element( &sen.to_file_name(index) );
                sen_image.set_id( &format!("bit_{}", index) );
                onsen_mark_sen.append_child(&sen_image).unwrap();
            } else {
                // 変更
                let sen_image: HtmlImageElement = js::DOM::get_element_by_id( &format!("bit_{}", index) );
                sen_image.set_src( &sen.to_file_name(index) );
            }
        });

        if first_render {
            // body の最後で読んでねと書いてあったけど、ここでいいっぽい
            js::fitty("#title_text");
        }
    }
}

// index.html から呼ばれる Model
pub struct MainModel {
    link: ComponentLink<Self>,
    page: Box<dyn PageTrait>,
}
impl Component for MainModel {
    type Message = Message;
    type Properties = ();

    // 作成
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            page: Box::new(QuastionPage::new()),
        }
    }

    // callback とかで Message を処理する
    fn update(&mut self, message: Self::Message) -> ShouldRender {
        self.page.as_mut().update(message);

        true
    }

    // プロパティが異なる場合以外は、常に false を返す必要がある
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    // 表示
    fn view(&self) -> Html {
        self.page.as_ref().view(&self.link)
    }

    // 描画後
    fn rendered(&mut self, first_render: bool) {
        self.page.as_mut().rendered(first_render);
    }
}
