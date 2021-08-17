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

/// イベントメッセージ
pub enum Message {
    // parent message
    ChangeToQuastionPage(String),
    ChangeToSelectPage,
    None,

    // top page
    TouchStart(web_sys::TouchEvent),
    TouchMove(web_sys::TouchEvent),
    TouchEnd,
    TouchStartBackSen,
    TouchStartFrontSen,

    // select page
    StageSelect(i32),
    StageBack,
    StageNext,
    StageEnter,
    StageClose,
    StageHint,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct OnsenName {
    name: String,

    #[serde(default)]
    roma: String,
}
impl OnsenName {
    /// roma をひらがなに変換して name に入れる
    fn japanification(&mut self) {
        // char += 一文字とる、if char == "" :char_index = i
        // romaji::convert する、!= だと、name に push、char を空にする
        // char が 5 文字になる or 残っている状態で最後までいくと、char_index+1 まで戻る、char_index は未処理文字とする
        let mut iter = self.roma.chars();
        let mut string = "".to_string();
        let mut char_index: usize = 0;
        let mut i = 0;
        self.name = "".to_string();
        loop {
            let mut char_opt = iter.next();
            if 4 <= string.len() {
                js::console_log!("skipped unconverted roma [{}].", string);
                i = char_index + 1;
                iter = self.roma.chars();
                char_opt = iter.nth(i);

                self.name.push_str(&romaji::convert::katakana_to_hiragana(
                    Self::special_katakata_map( &string.chars().nth(0).unwrap().to_string() )
                ));
                string.clear();
            } else if char_opt == None {
                if !string.is_empty() {
                    js::console_log!("ended unconverted roma [{}].", string);
                    self.name.push_str( &romaji::convert::katakana_to_hiragana(Self::special_katakata_map(&string)) );
                }
                break;
            }

            if string.is_empty() {
                char_index = i;
            }
            string.push( char_opt.unwrap() );

            js::console_log!("char: {:?}, string: {}, name: {}", char_opt, string, self.name);
            if string == "n" {
                i += 1;
                continue;
            }

            let mut katakanas = romaji::convert::romaji_to_katakana(string.clone());
            if katakanas != string {
                // 促音だけが変換される場合があるので確認する
                // "nn" で促音が入る場合があるので消す
                if "nn" == string {
                    katakanas = "ン".to_string();
                }
                let mut hiragana = "".to_string();
                for katakana in katakanas.chars() {
                    if katakana.is_ascii_alphabetic() {
                        hiragana.clear();
                        break;
                    }
                    hiragana.push_str( &romaji::convert::katakana_to_hiragana(katakana.to_string()) );
                }

                if !hiragana.is_empty() {
                    // 促音もASCIIも含まれていなかった
                    self.name.push_str(&hiragana);
                    string.clear();
                }
            }
            i += 1;
        }

        js::console_log!("onsen name is [{}]", self.name);
    }

    /// roma を適当につなげただけだと余る文字が出てくるので、特別にカタカナへ変換する (元の英語の発音に基づきかつ子音)
    fn special_katakata_map(romaji: &str) -> String {
        match romaji {
            "d" => "ド",    // an 'd'
            "f" => "フ",    // o 'f' f
            "n" => "ン",    // a 'n' d
            "r" => "ル",    // o 'r'
            "t" => "ト",    // no 't'
            _ => "",
        }.to_string()
    }
}
impl Default for OnsenName {
    fn default() -> Self {
        Self {
            name: "？？？".to_string(),
            roma: "".to_string(),
        }
    }
}
impl std::fmt::Display for OnsenName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}泉", self.name)
    }
}
impl std::ops::AddAssign<String> for OnsenName {
    fn add_assign(&mut self, rhs: String) {
        self.roma += &rhs;
    }
}


// 問題
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct OnsenStatus {
    is_clear: bool,
    temperature: i32,
    sen: sen::SenManager,
    info: String,
    onsen_name: OnsenName,

    #[serde(default)]
    key: String,
}
impl OnsenStatus {
    fn new(key: &str) -> Self {
        Self::default().set_key(key)
    }

    fn set_key(mut self, key: &str) -> Self {
        self.key = key.to_string();
        self
    }

    fn get_onsen_key_from_stage(stage_level: i32, stage_number: i32) -> String {
        format!("stage_{}_{}", stage_level, stage_number)
    }

    fn get_onsen_status_from_name<'a>(name: &str) -> &'a Self {
        &Resource::onsen_status_manager().onsen_status_list[name]
    }

    fn get_init_onsen_status<'a>() -> &'a Self {
        Self::get_onsen_status_from_name(&Resource::user_storage().init_onsen_key)
    }

    fn get_onsen_status<'a>(stage_level: i32, stage_number: i32) -> &'a Self {
        Self::get_onsen_status_from_name( &Self::get_onsen_key_from_stage(stage_level, stage_number) )
    }

    fn get_popup_html(&self, link: &ComponentLink<MainModel>) -> Html {
        let tweet_url = format!("https://twitter.com/intent/tweet?text=On！Sen！%0d{}に入ったよ！%0d&hashtags=onsen", &self.onsen_name);

        html! {
            <div id="stage_detail_div">
                <div id="stage_name">
                    { &self.onsen_name }
                </div>
                <div id="stage_detail">
                    { "★☆☆" }
                </div>
                <div id="stage_external_icon">
                    <button>
                        <a href={ tweet_url }>
                            <img class="stage_link_image" src="/resource/Twitter_social_icons_rounded_square_blue.png" />
                        </a>
                    </button>
                    <button>
                        <a href={ format!("/rewarded_ad?stage_name={}", self.onsen_name) }>
                            <img class="stage_link_image" src="/resource/Tips-icon.png" />
                        </a>
                    </button>
                    <button>
                        <a href="https://www.youtube.com/channel/UCVxzptdb4sw84z2rTh_1AJQ">
                            <img class="stage_link_image" src="/resource/youtube_social_squircle_red.png" />
                        </a>
                    </button>
                </div>
                <div id="stage_navigation">
                    <button type="button" ontouchend=link.callback(|event| Message::StageBack)>
                        <img src="/resource/navigation_back.png" />
                    </button>
                    <button type="button" ontouchend=link.callback(|event| Message::StageEnter)>
                        <img src="/resource/navigation_go.png" />
                    </button>
                    <button type="button" ontouchend=link.callback(|event| Message::StageNext)>
                        <img src="/resource/navigation_next.png" />
                    </button>
                </div>
            </div>
        }
    }

    fn update_popup(&self, message: &Message) {
        match message {
            Message::StageHint => {
                js::fs::get_sync_request(&format!("/rewarded_ad?stage_name={}", self.onsen_name));
            },
            _ => (),
        }
    }
}

// 問題を管理するクラス
#[derive(Debug, Deserialize, Serialize)]
struct OnsenStatusManager {
    pub onsen_status_list: HashMap<String, OnsenStatus>,
}
impl OnsenStatusManager {
    const FILE_PATH: &'static str = "/data/quastions.json";

    fn from_file() -> Self {
        let file = js::fs::open( Self::FILE_PATH.to_string() );
        let reader = std::io::BufReader::new(file);

        match serde_json::from_reader::<_, Self>(reader) {
            Ok(onsen_status_list) => {
                onsen_status_list
            },
            Err(e) => {
                panic!("invalid quastions.json E:{:?}", e);
            }
        }
    }

    fn set_onsen_data(&mut self, key: &str, onsen_data: &OnsenStatus) {
        *self.onsen_status_list.get_mut(key).unwrap() = onsen_data.clone();
    }
}

// セーブデータを管理するクラス
#[derive(Debug, Deserialize, Serialize)]
struct UserStorage {
    init_onsen_key: String,
    onsen_status: OnsenStatusManager,
}
impl UserStorage {
    const STORAGE_DATA_NAME: &'static str = "save_data.json";

    // UserStorage を作成するために初期値用
    fn new(init_onsen_key: String) -> Self {
        Self {
            init_onsen_key,
            onsen_status: OnsenStatusManager::from_file(),
        }
    }

    // UserStorage へ保存
    fn save_data(&self) {
        js::fs::write_storage(
            Self::STORAGE_DATA_NAME.to_string(),
            serde_json::to_string( &self ).unwrap(),
        );
    }
}
impl Default for UserStorage {
    // UserStorage から読み込み。無いなら作成
    fn default() -> Self {
        let file = js::fs::read_storage( Self::STORAGE_DATA_NAME.to_string() );

        match file {
            Some(file) => {
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader::<_, Self>(reader) {
                    Ok(data) => {
                        data
                    },
                    Err(e) => {
                        panic!("invalid {} E:{:?}", Self::STORAGE_DATA_NAME, e);
                    }
                }
            },
            None => {
                // 初期起動なので作成する
                let data = Self::new("top".to_string());
                data.save_data();
                data
            },
        }
    }
}

/// 一度だけ読み込む必要があるリソースを保持するクラス
struct Resource {
    onsen_status_manager: Option<OnsenStatusManager>,   // ステージ情報
    user_storage: Option<UserStorage>,                  // ユーザー情報
}
impl Resource {
    fn get_onsen_status_manager(&mut self) -> &OnsenStatusManager {
        if self.onsen_status_manager.is_none() {
            self.onsen_status_manager = Some(OnsenStatusManager::from_file());
        }

        self.onsen_status_manager.as_ref().unwrap()
    }

    fn get_user_storage(&mut self) -> &mut UserStorage {
        if self.user_storage.is_none() {
            self.user_storage = Some(UserStorage::default());
        }

        self.user_storage.as_mut().unwrap()
    }

    pub fn onsen_status_manager<'a>() -> &'a OnsenStatusManager {
        unsafe{ RESOURCE.get_onsen_status_manager() }
    }

    pub fn user_storage<'a>() -> &'a mut UserStorage {
        unsafe{ RESOURCE.get_user_storage() }
    }
}
static mut RESOURCE: Resource = Resource {
    onsen_status_manager: None,
    user_storage: None,
};

/// ページ遷移用の雛形
trait PageTrait {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html;

    fn update(&mut self, _message: Message) -> Message { Message::None }
    fn rendered(&mut self, _first_render: bool) {}
}

struct SelectPage {
    stage_number: i32,
    stage_level: i32,
    saved_onsen_data: &'static OnsenStatus,
}
impl SelectPage {
    fn new() -> Self {
        Self {
            stage_number: -1,
            stage_level: 0,
            saved_onsen_data: OnsenStatus::get_init_onsen_status()
        }
    }
}
impl PageTrait for SelectPage {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        html!{
            <div class="select_container" id="grand_parent_node">
                <div class="container_item_header">
                    /*{ "banner_area" }*/
                </div>
                <div class="select_container_item_content">
                    <div class="stage_3">
                        <img class="is_clear_onsen_mark" src="/resource/mark_offsen.png" alt="onsen_mark"
                            ontouchend=link.callback(|event| Message::StageSelect(3))
                        />
                    </div>
                    <div class="stage_2">
                        <img class="is_clear_onsen_mark" src="/resource/mark_offsen.png" alt="onsen_mark"
                            ontouchend=link.callback(|event| Message::StageSelect(2))
                        />
                    </div>
                    <div class="stage_1">
                        <img class="is_clear_onsen_mark" src="/resource/mark_offsen.png" alt="onsen_mark"
                            ontouchend=link.callback(|event| Message::StageSelect(1))
                        />
                    </div>
                    <div class="stage_0">
                        <img class="is_clear_onsen_mark" src="/resource/mark_offsen.png" alt="onsen_mark"
                            ontouchend=link.callback(|event| Message::StageSelect(0))
                        />
                    </div>

                    { self.saved_onsen_data.get_popup_html(link) }
                </div>
                <div class="container_item_footer">
                    /*{ "banner_area" }*/
                </div>
                <img id="onsen_select_map"
                    src="/resource/onsen_select_map.png"
                    ontouchend=link.callback(|event| Message::StageClose)
                />
            </div>
        }
    }

    fn update(&mut self, message: Message) -> Message {
        self.saved_onsen_data.update_popup(&message);

        match message {
            Message::StageBack => {
                
            },
            Message::StageNext => {
                
            },
            Message::StageEnter => {
                return Message::ChangeToQuastionPage(
                    OnsenStatus::get_onsen_key_from_stage(self.stage_level, self.stage_number)
                );
            },
            Message::StageSelect(stage_number) => {
                let stage_detail_div: HtmlElement = js::dom::get_element_by_id("stage_detail_div").unwrap();
                stage_detail_div.class_list().remove_1("stage_detail_hide_popup").unwrap();
                stage_detail_div.class_list().add_1("stage_detail_show_popup").unwrap();

                js::fitty("#stage_name");
                js::fitty("#stage_detail");

                self.stage_number = stage_number;
                self.saved_onsen_data = OnsenStatus::get_onsen_status(self.stage_level, self.stage_number);
                
                js::console_log!("to stage {}", stage_number);
            },
            Message::StageClose => {
                if -1 != self.stage_number {
                    let stage_detail_div: HtmlElement = js::dom::get_element_by_id("stage_detail_div").unwrap();
                    stage_detail_div.class_list().remove_1("stage_detail_show_popup").unwrap();
                    stage_detail_div.class_list().add_1("stage_detail_hide_popup").unwrap();
                    self.stage_number = -1;
                }
            },
            _ => (),
        };

        Message::None
    }
}

// 問題出題ページは Topページ も兼ねている
struct QuastionPage {
    sen_op: sen::SenOpManager,
    quastion: &'static OnsenStatus,
    now_status: OnsenStatus,
    cursor_image: Option<HtmlImageElement>,
}
impl QuastionPage {
    fn new(name: &str) -> Self {
        let mut own = Self {
            sen_op: sen::SenOpManager::new( vec![sen::SenOp::Off, sen::SenOp::On, sen::SenOp::Or, sen::SenOp::And, sen::SenOp::Not], Some(1) ),
            quastion: &Resource::onsen_status_manager().onsen_status_list[name],
            now_status: OnsenStatus::new(name),
            cursor_image: None,
        };
        own.load_quastion(name);

        own
    }

    // 問題の読み込みと現在の状態の初期化
    fn load_quastion(&mut self, name: &str) {
        self.quastion = OnsenStatus::get_onsen_status_from_name(name);
        self.now_status.sen.deep_copy(&self.quastion.sen);
    }
}
impl PageTrait for QuastionPage {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        html! {
            <div class="top_container" id="grand_parent_node">
                <div class="container_item_header">
                    /*{ "banner_area" }*/
                </div>
                <div class="container_item_title">
                    <div id="title_text">
                        <div>
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

    fn update(&mut self, message: Message) -> Message {
        match message {
            Message::TouchStart(event) => {
                // ドラッグ中の温泉マークを作成
                let x = event.touches().get(0).unwrap().client_x();
                let y = event.touches().get(0).unwrap().client_y();
                let parent_image: HtmlImageElement = js::dom::get_element_by_id("onsen_mark").unwrap();
                
                let image: HtmlImageElement = js::dom::make_img_element( &parent_image.current_src() );
                image.set_id("temp_onsen_mark");
                image.style().set_property("left", &format!("{}px", x) ).unwrap();
                image.style().set_property("top", &format!("{}px", y) ).unwrap();
                self.cursor_image = Some(image);

                js::dom::get_element_by_id::<HtmlElement>("grand_parent_node").unwrap()
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
                    // event から座標が取れなかったので(TouchEndだからカーソルの座標がない？)、TouchMove で動かしてた css から取る
                    let left: String = image.style().get_property_value("left").unwrap();
                    x = left.split_at( left.len() - 2 ).0.parse::<f64>().unwrap();
                    let top: String = image.style().get_property_value("top").unwrap();
                    y = top.split_at( top.len() - 2 ).0.parse::<f64>().unwrap();
                }

                let sen_op_top = self.sen_op.get_top();
                let mut new_op = self.now_status.sen.operation(&self.sen_op.get_top(), &mut |(index, _sen)| {
                    let sen_image: HtmlImageElement = js::dom::get_element_by_id(&format!("bit_{}", index)).unwrap();
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
                self.now_status.temperature = self.now_status.sen.get_number();
                self.now_status.onsen_name += sen_op_top.to_string().to_lowercase();

                // ドラッグ中の温泉マークを削除
                if let Some(image) = self.cursor_image.as_mut() {
                   image.remove();
                   self.cursor_image = None;
                }

                js::console_log!("{:?} / {:?}", self.sen_op, self.now_status);

                // 正解と同じ温度なら保存して、ステージ選択画面へ
                if self.now_status.temperature == self.quastion.temperature {
                    self.now_status.is_clear = true;
                    self.now_status.onsen_name.japanification();
                    Resource::user_storage().onsen_status.set_onsen_data(
                        &self.now_status.key,
                        &self.now_status
                    );
                    Resource::user_storage().save_data();
                    return Message::ChangeToSelectPage;
                }
            },
            Message::TouchStartBackSen => self.sen_op.prev(),
            Message::TouchStartFrontSen => self.sen_op.next(),
            _ => (),
        }

        Message::None
    }

    fn rendered(&mut self, first_render: bool) {
        // onsen_mark_sen に動的にぶら下げるので sen はここで表示
        let onsen_mark_sen: HtmlElement = js::dom::get_element_by_id("onsen_mark_sen").unwrap();
        self.now_status.sen.for_each(|(index, sen)|{
            // 存在すれば上書き、なければ作成
            match js::dom::get_element_by_id::<HtmlImageElement>( &format!("bit_{}", index) ) {
                Some(sen_image) => {
                    sen_image.set_src( &sen.to_file_name(index) );
                },
                None => {
                    let sen_image = js::dom::make_img_element( &sen.to_file_name(index) );
                    sen_image.set_id( &format!("bit_{}", index) );
                    onsen_mark_sen.append_child(&sen_image).unwrap();
                },
            }
        });

        if first_render {
            // body の最後で読んでねと書いてあったけど、ここでいいっぽい
            js::fitty("#title_text");
        }
    }
}

struct PageManager {
    page: Box<dyn PageTrait>,
}
impl PageManager {
    fn new() -> Self {
        Self {
            page: Box::new(QuastionPage::new(&Resource::user_storage().init_onsen_key)),
        }
    }

    fn update(&mut self, message: Message) -> ShouldRender {
        match self.page.as_mut().update(message) {
            Message::ChangeToQuastionPage(name) => {
                Resource::user_storage().init_onsen_key = name;
                Resource::user_storage().save_data();

                js::console_log!("{:?}", js::time::now());
                js::dom::super_redirect( "/index.html" );
            },
            Message::ChangeToSelectPage => {
                self.page = Box::new(SelectPage::new());
            },
            _ => (),
        };
                
        true
    }

    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        self.page.as_ref().view(link)
    }

    // 描画後
    fn rendered(&mut self, first_render: bool) {
        self.page.as_mut().rendered(first_render);
    }
}

// index.html から呼ばれる Model
pub struct MainModel {
    link: ComponentLink<Self>,
    page_manager: PageManager,
}
impl Component for MainModel {
    type Message = Message;
    type Properties = ();

    // 作成
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            page_manager: PageManager::new(),
        }
    }

    // callback とかで Message を処理する
    fn update(&mut self, message: Self::Message) -> ShouldRender {
        self.page_manager.update(message)
    }

    // プロパティが異なる場合以外は、常に false を返す必要がある
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    // 表示
    fn view(&self) -> Html {
        self.page_manager.view(&self.link)
    }

    // 描画後
    fn rendered(&mut self, first_render: bool) {
        self.page_manager.rendered(first_render);
    }
}
