use yew::prelude::*;
use web_sys::*;

use crate::sen;
use crate::js;

pub enum Message {
    TouchStart,
    TouchMove,
    TouchEnd,
}

/// ページ遷移用の雛形
trait PageTrait {

    fn view(&self, link: &ComponentLink<MainModel>) -> Html;

    fn update(&mut self, _message: Message) {}
    fn rendered(&mut self, _first_render: bool) {}
}

// 問題出題ページは Topページ も兼ねている
pub struct QuastionPage {
    sen_kind: sen::SenOp,
}
impl PageTrait for QuastionPage {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        let back_logic = format!("{}", "Off");
        let logic = format!("{:?}", self.sen_kind);
        let front_logic = format!("{}", "Or");

        html! {
            <div class="container">
                <div class="container_item_header">
                    /*{ "banner_area" }*/
                </div>
                <div class="container_item_title">
                    <div id="title_text">
                        <ruby data-ruby={ front_logic }>
                            <rb>{ logic }</rb>
                            <rt class="rt_upper">{ back_logic }</rt>
                        </ruby>
                        { "Sen" }
                        <img id="onsen_mark" src="/resource/mark_onsen.png" alt="onsen_mark"
                            ontouchstart=link.callback(|_| Message::TouchStart)
                            ontouchend=link.callback(|_| Message::TouchEnd)
                            ontouchmove=link.callback(|_| Message::TouchMove) />
                    </div>
                </div>
                <div class="container_item_content">
                    <img id="open_air_bath" src="/resource/onsen_ofuro_noone_bg.png" alt="open_air_bath" />
                    <div id="wood_kanban_div">
                        <img id="wood_kanban" src="/resource/wood_kanban.png" alt="wood_kanban" />
                        <div id="wood_kanban_text_div">
                            <div id="wood_kanban_text">{ "42℃" }</div>
                        </div>
                    </div>
                    <div id="ondokei_div">
                        <img id="ondokei" src="/resource/ondokei.png" alt="ondokei" />
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
            Message::TouchStart => {
                js::console_log!("do Touch!");
            },
            Message::TouchMove => {
                js::console_log!("Touch move");
            },
            Message::TouchEnd => {
                js::console_log!("Touch end!");
            },
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let onsen_mark_sen: HtmlElement = js::DOM::get_element_by_id("onsen_mark_sen");
            onsen_mark_sen.append_child(&js::DOM::make_img_element("/resource/mark_small_sen_0b00.png")).unwrap();
            onsen_mark_sen.append_child(&js::DOM::make_img_element("/resource/mark_sen_0b00.png")).unwrap();
            onsen_mark_sen.append_child(&js::DOM::make_img_element("/resource/mark_small_sen_0b00.png")).unwrap();
    
            // body の最後で読んでねと書いてあったけど、ここでいいっぽい
            js::fitty("#title_text");
        }
    }
}
impl QuastionPage {
    fn new() -> Self {
        Self {
            sen_kind: sen::SenOp::On,
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
