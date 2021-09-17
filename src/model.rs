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
#[derive(Clone)]
pub enum Message {
    // parent message
    ChangeToQuastionPage(String),
    ChangeToSelectPage,
    None,

    // quastion page
    TouchStart(web_sys::TouchEvent),
    TouchMove(web_sys::TouchEvent),
    TouchEnd,
    TouchStartBackSen,
    TouchStartFrontSen,
    BlinkAnimationEnd,
    OpenDoor,
    OpenedDoor,

    // select page
    StageSelect(i32),
    StageBack,
    StageNext,
    StageEnter,
    StageClose,
    StageTweet,
    StageHint,
    StageYoutube,
    AllComplete,
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

            js::console_log!("i: {}, char: {:?}, string: {}, name: {}", i, char_opt, string, self.name);
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
    is_using_onsen: bool,
    is_lower_border: bool,
    use_hint: bool,

    ops_border: i32,

    temperature: i32,
    sen: sen::SenManager,
    #[serde(default)]
    ops_count: i32,

    info: String,
    onsen_name: OnsenName,

    key: String,
    back_key: String,
    next_key: String,
}
impl OnsenStatus {
    /// インスタンス作成
    fn new(key: &str) -> Self {
        Resource::onsen_status_manager().onsen_status_list[key].clone()
            .set_temperature(0)
            .set_use_hint(Resource::user_storage().onsen_status.onsen_status_list[key].use_hint)
    }

    /// 初期状態にする
    fn init(&mut self) {
        *self = Resource::onsen_status_manager().onsen_status_list[&self.key].clone();
        self.use_hint = Resource::user_storage().onsen_status.onsen_status_list[&self.key].use_hint;
    }

    /// user storage に保存されている状態にする
    fn init_from_user_storage(&mut self) {
        *self = Resource::user_storage().onsen_status.onsen_status_list[&self.key].clone();
    }

    /// クリアしてるかどうか
    fn is_cleared(&self) -> bool {
        self.is_clear || self.is_lower_border || self.is_using_onsen
    }

    /// ☆をコンプリートしてるかどうか
    fn is_complete(&self) -> bool {
        self.is_clear && self.is_lower_border && self.is_using_onsen
    }

    /// 温度の設定
    fn set_temperature(mut self, temperature: i32) -> Self {
        self.temperature = temperature;
        self
    }

    fn set_use_hint(mut self, use_hint: bool) -> Self {
        self.use_hint = use_hint;
        self
    }

    fn use_hint_and_save(&mut self) {
        if Resource::user_storage().onsen_status.onsen_status_list[&self.key].is_cleared() {
            // 過去にクリアしていると初期化しない
            self.init_from_user_storage();
        } else {
            // json から初期化
            self.init();
            self.sen.init();
        }
        self.use_hint = true;

        Resource::user_storage().onsen_status.set_onsen_data(
            &self.key,
            &self
        );
        Resource::user_storage().save_data();
    }

    /// 前の ステージキー を返す
    fn back_key(&self) -> String {
        self.back_key.clone()
    }

    /// 次の ステージキー を返す
    fn next_key(&self) -> String {
        self.next_key.clone()
    }

    /// stage 番号から ステージキー を返す
    fn get_onsen_key_from_stage(stage_level: i32, stage_number: i32) -> String {
        format!("stage_{}_{}", stage_level, stage_number)
    }

    /// stage 名から OnsenStatus を返す
    fn get_onsen_status_from_name<'a>(name: &str) -> &'a Self {
        &Resource::user_storage().onsen_status.onsen_status_list[name]
    }

    /// 初期の OnsenStatus を返す
    fn get_init_onsen_status<'a>() -> &'a Self {
        Self::get_onsen_status_from_name(&Resource::user_storage().init_onsen_key)
    }

    /// stage 番号から OnsenStatus を返す
    fn get_onsen_status<'a>(stage_level: i32, stage_number: i32) -> &'a Self {
        Self::get_onsen_status_from_name( &Self::get_onsen_key_from_stage(stage_level, stage_number) )
    }

    /// ポップアップのための HTML を返す
    fn get_popup_html(&self, link: &ComponentLink<MainModel>, is_stage_view: bool) -> Html {
        let tweet_url = if self.is_cleared() {
            format!("{}に入ったよ！", &self.onsen_name)
        } else {
            format!("{}に入りたい！", &self.info)
        };
        let tweet_url = format!("https://twitter.com/intent/tweet?text=On！Sen！%0d{}%0d&hashtags=onsen", tweet_url);
        
        let hint_button = if self.use_hint {
            html! {
                <div class="hint_icon" id="hint_onsen_mark_sen"></div>
            }
        } else {
            html! {
                <button class="hint_icon">
                    <a href={ format!("/rewarded_ad?stage_name={}", self.onsen_name) }>
                        <img src="/resource/image/Tips-icon.png" />
                    </a>
                </button>
            }
        };

        html! {
            <div id="stage_detail_div">
                <img id="stage_detail_background" src="/resource/image/wood_kanban_tate5.png" />
                <div id="stage_name_div">
                    <div id="stage_name">
                        { &self.onsen_name }
                    </div>
                </div>
                <div id="stage_detail">
                    { if self.is_clear {"★"} else {"☆"} }{ if self.is_using_onsen {"★"} else {"☆"} }{ if self.is_lower_border {"★"} else {"☆"} }
                </div>
                { if is_stage_view { self.get_move_navigation_html(link) } else { self.get_clear_navigation_html(link) } }
                <div id="stage_external_icon">
                    <button class="tweet_icon"
                        ontouchend=link.callback(|_| Message::StageTweet)
                    >
                        <a target="_blank" href={ tweet_url }>
                            <img src="/resource/image/Twitter_social_icons_rounded_square_blue.png" />
                        </a>
                    </button>
                    { hint_button }
                    <button class="youtube_icon"
                        ontouchend=link.callback(|_| Message::StageYoutube)
                    >
                        <a target="_blank" href="https://www.youtube.com/channel/UCVxzptdb4sw84z2rTh_1AJQ?sub_confirmation=1">
                            <img src="/resource/image/youtube_social_squircle_red.png" />
                        </a>
                    </button>
                </div>
                <div id="stage_info">
                    { &self.info }
                </div>
            </div>
        }
    }

    fn get_navigation_button(&self, link: &ComponentLink<MainModel>, key: String, text: &str, message: Message) -> Html {
        if key == self.key {
            // 無表示だとレイアウトが崩れるので、空ボタンを返す
            return html!{
                <button type="button" ontouchend=link.callback(move |event| message.clone())>{ "　　" }</button>
            };
        }

        html! {
            <button type="button" ontouchend=link.callback(move |event| message.clone())>
                { text }
            </button>
        }
    }

    /// 移動するための ナビゲーション を返す
    fn get_move_navigation_html(&self, link: &ComponentLink<MainModel>) -> Html {
        html! {
            <div id="stage_navigation">
                { self.get_navigation_button(link, self.back_key(), "前", Message::StageBack) }
                <button type="button" ontouchend=link.callback(|event| Message::StageEnter)>
                    { "入" }
                </button>
                { self.get_navigation_button(link, self.next_key(), "次", Message::StageNext) }
            </div>
        }
    }

    /// クリアしたときの ナビゲーション を返す
    fn get_clear_navigation_html(&self, link: &ComponentLink<MainModel>) -> Html {
        html! {
            <div id="stage_navigation">
                <button type="button" ontouchend=link.callback(|event| Message::StageBack)>
                    { "戻" }
                </button>
                <button type="button" ontouchend=link.callback(|event| Message::StageEnter)>
                    { "再" }
                </button>
                { self.get_navigation_button(link, self.next_key(), "次", Message::StageNext) }
            </div>
        }
    }

    fn is_show_popup() -> bool {
        if let Some(stage_detail_div) = js::dom::get_element_by_id::<HtmlElement>("stage_detail_div") {
            return stage_detail_div.class_list().contains("stage_detail_show_popup");
        }

        false
    }

    /// ポップアップを表示する
    fn show_popup() {
        if let Some(stage_detail_div) = js::dom::get_element_by_id::<HtmlElement>("stage_detail_div") {
            stage_detail_div.class_list().remove_1("stage_detail_hide_popup").unwrap();
            stage_detail_div.class_list().add_1("stage_detail_show_popup").unwrap();
        }
    }

    /// ポップアップを非表示にする
    fn hide_popup() {
        if let Some(stage_detail_div) = js::dom::get_element_by_id::<HtmlElement>("stage_detail_div") {
            stage_detail_div.class_list().remove_1("stage_detail_show_popup").unwrap();
            stage_detail_div.class_list().add_1("stage_detail_hide_popup").unwrap();
        }
    }

    /// ポップアップのアップデート
    fn update_popup(&self, message: &Message) {
        match message {
            Message::StageHint => {
                Resource::user_storage().init_onsen_key = self.key.to_string();
                Resource::user_storage().save_data();
                js::fs::get_sync_request(&format!("/rewarded_ad?stage_name={}", self.onsen_name));
            },
            _ => (),
        }
    }

    fn rendered(&self) {
        if let Some(hint_onsen_mark_sen) = js::dom::get_element_by_id::<HtmlElement>("hint_onsen_mark_sen") {
            Resource::onsen_status_manager().onsen_status_list[&self.key].sen.for_each(|(index, sen)|{
                // 存在すれば上書き、なければ作成
                match js::dom::get_element_by_id::<HtmlImageElement>( &format!("hint_bit_{}", index) ) {
                    Some(sen_image) => {
                        sen_image.set_src( &sen.to_file_name(index) );
                    },
                    None => {
                        let sen_image = js::dom::make_img_element( &sen.to_file_name(index) );
                        sen_image.set_id( &format!("hint_bit_{}", index) );
                        sen_image.set_class_name("hint_sen");

                        hint_onsen_mark_sen.append_child(&sen_image).unwrap();
                    },
                }
            });
        }
    }
}

// 問題を管理するクラス
#[derive(Debug, Deserialize, Serialize)]
struct OnsenStatusManager {
    pub level_list: Vec<i32>,
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
    init_stage_level: i32,
    onsen_status: OnsenStatusManager,
}
impl UserStorage {
    const STORAGE_DATA_NAME: &'static str = "save_data.json";

    // UserStorage を作成するために初期値用
    fn new(init_onsen_key: String) -> Self {
        Self {
            init_onsen_key,
            init_stage_level: 0,
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

use strum::IntoEnumIterator;
#[derive(Clone, Copy, Debug, strum_macros::EnumIter, PartialEq)]
enum SoundKind {
    ButtonUp,
    DownTemp,
    Sen01,
    Sen10,
    Sen11,
    StageEnter,
    UpTemp,
    BgmStageDefault,
    BgmStageTop,
    BgmStageLevel1,
    BgmStageLevel2,
}
impl SoundKind {
    fn get_id(&self) -> &str {
        match *self {
            Self::ButtonUp => "se_button_up.mp3",
            Self::DownTemp => "se_down_temp.mp3",
            Self::Sen01 => "se_sen_0b01.wav",
            Self::Sen10 => "se_sen_0b10.wav",
            Self::Sen11 => "se_sen_0b11.wav",
            Self::StageEnter => "se_stage_enter.mp3",
            Self::UpTemp => "se_up_temp.ogg",
            Self::BgmStageDefault => "bgm_stage_default.wav",
            Self::BgmStageTop => "bgm_stage_top.wav",
            Self::BgmStageLevel1 => "bgm_stage_level_1.wav",
            Self::BgmStageLevel2 => "bgm_stage_level_2.wav",
        }
    }

    fn get_audio_html(&self) -> Html {
        html! {
            <audio id={ self.get_id() } preload="auto" src={ format!("/resource/sound/{}", self.get_id()) } />
        }
    }
}

#[derive(Debug, Default)]
struct Sound {
    playing_sound: Vec<SoundKind>,
}
impl Sound {
    fn is_playing(&self, sound_kind: &SoundKind) -> bool {
        self.playing_sound.iter().any(|sk| sk == sound_kind)
    }

    fn play_loop(&mut self, sound_kind: SoundKind) {
        if self.is_playing(&sound_kind) {
            return;
        }

        if let Some(audio_element) = js::dom::get_element_by_id::<HtmlMediaElement>(sound_kind.get_id()) {
            audio_element.set_loop(true);
        }
        self.play(sound_kind);
    }
    fn play(&mut self, sound_kind: SoundKind) {
        if let Some(audio_element) = js::dom::get_element_by_id::<HtmlMediaElement>(sound_kind.get_id()) {
            if !self.is_playing(&sound_kind) {
                self.playing_sound.push(sound_kind.clone());
            }

            audio_element.set_current_time(0.0);
            let _ = audio_element.play();
            
            js::console_log!("play {:?}", sound_kind.get_id());
        }
    }

    fn pause(&mut self, sound_kind: SoundKind) {
        if let Some(index) = self.playing_sound.iter().position(|sk| *sk == sound_kind) {
            self.playing_sound.remove(index);
        }

        if let Some(audio_element) = js::dom::get_element_by_id::<HtmlMediaElement>(sound_kind.get_id()) {
            if !audio_element.ended() {
                audio_element.set_current_time(0.0);
                let _ = audio_element.pause();
            }
            js::console_log!("pause {:?}", sound_kind.get_id());
        }
    }

    fn pause_all(&mut self) {
        while let Some(sk) = self.playing_sound.pop() {
            self.pause(sk);
        }
    }

    fn set_volume(&self, sound_kind: SoundKind, volume: f64) {
        if let Some(audio_element) = js::dom::get_element_by_id::<HtmlMediaElement>(sound_kind.get_id()) {
            audio_element.set_volume(volume);
        }
    }
}
impl PageTrait for Sound {
    fn view(&self, _link: &ComponentLink<MainModel>) -> Html {
        html!{
            <>
                { for SoundKind::iter().map(|sound_kind| sound_kind.get_audio_html()) }
            </>
        }
    }
}

/// 一度だけ読み込む必要があるリソースを保持するクラス
struct Resource {
    onsen_status_manager: Option<OnsenStatusManager>,   // ステージ情報
    user_storage: Option<UserStorage>,                  // ユーザー情報
    sound: Option<Sound>,                               // 音
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

    fn get_sound(&mut self) -> &mut Sound {
        if self.sound.is_none() {
            self.sound = Some(Sound::default());
        }

        self.sound.as_mut().unwrap()
    }

    pub fn onsen_status_manager<'a>() -> &'a OnsenStatusManager {
        unsafe{ RESOURCE.get_onsen_status_manager() }
    }

    pub fn user_storage<'a>() -> &'a mut UserStorage {
        unsafe{ RESOURCE.get_user_storage() }
    }

    pub fn sound<'a>() -> &'a mut Sound {
        unsafe{ RESOURCE.get_sound() }
    }
}
static mut RESOURCE: Resource = Resource {
    onsen_status_manager: None,
    user_storage: None,
    sound: None,
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
        let own = Self {
            stage_number: -1,
            stage_level: Resource::user_storage().init_stage_level,
            saved_onsen_data: OnsenStatus::get_init_onsen_status()
        };
        own.play_bgm();

        own
    }

    fn play_bgm(&self) {
        let bgm_kind = match self.stage_level+1 {
            2 => SoundKind::BgmStageLevel2,
            _ => SoundKind::BgmStageLevel1,
        };

        Resource::sound().set_volume(bgm_kind, 0.5);
        Resource::sound().play_loop(bgm_kind);
    }

    const NOT_CLEAR_ONSEN_MARK: &'static str = "/resource/image/mark_offsen.png";
    const CLEAR_ONSEN_MARK: &'static str = "/resource/image/mark_orsen.png";
    const ALL_CLEAR_ONSEN_MARK: &'static str = "/resource/image/mark_onsen.png";
}
impl PageTrait for SelectPage {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        // ステージ毎の簡易表示(クリア情報と遷移先)
        let mut select_container_item_content_html = vec![];
        let mut all_complete = true;
        for &stage_level in &Resource::onsen_status_manager().level_list {
            for stage_number in 0..4 {
                let temp_status = OnsenStatus::get_onsen_status(stage_level, stage_number);

                if all_complete {
                    all_complete = temp_status.is_complete();
                }
                if stage_level != self.stage_level {
                    continue
                }

                let clear_mark = if all_complete {
                    Self::ALL_CLEAR_ONSEN_MARK
                } else if temp_status.is_cleared() {
                    Self::CLEAR_ONSEN_MARK
                } else {
                    Self::NOT_CLEAR_ONSEN_MARK
                };

                select_container_item_content_html.push(html!{
                    <div class={ format!("stage_{}", stage_number) }>
                        <img class="is_clear_onsen_mark" src={ clear_mark } alt="onsen_mark"
                            ontouchend=link.callback( move |event| Message::StageSelect(stage_number) )
                        />
                    </div>
                });
            }
        }

        if all_complete && !OnsenStatus::is_show_popup() {
            js::console_log!("all complete");
            link.send_message(Message::AllComplete);
        }

        html!{
            <div class="select_container" id="grand_parent_node">
                <div class="container_item_header">
                    /*{ "banner_area" }*/
                </div>
                <div class="select_container_item_content">
                    { for select_container_item_content_html }
                    { self.saved_onsen_data.get_popup_html(link, true) }
                </div>
                <div class="container_item_footer">
                    /*{ "banner_area" }*/
                </div>
                <img id="onsen_select_map"
                    src="/resource/image/onsen_select_map.png"
                    ontouchend=link.callback(|event| Message::StageClose)
                />
            </div>
        }
    }

    fn update(&mut self, message: Message) -> Message {
        self.saved_onsen_data.update_popup(&message);

        match message {
            Message::StageEnter => {
                return Message::ChangeToQuastionPage(
                    self.saved_onsen_data.key.clone()
                );
            },
            Message::StageBack => {
                if self.saved_onsen_data.back_key() != self.saved_onsen_data.key {
                    js::console_log!("to stage {}", self.saved_onsen_data.back_key());
                    self.stage_number = i32::MAX;
                    self.saved_onsen_data = OnsenStatus::get_onsen_status_from_name(&self.saved_onsen_data.back_key());
                }
            },
            Message::StageNext => {
                if self.saved_onsen_data.next_key() != self.saved_onsen_data.key {
                    js::console_log!("to stage {}", self.saved_onsen_data.next_key());
                    self.stage_number = i32::MAX;
                    self.saved_onsen_data = OnsenStatus::get_onsen_status_from_name(&self.saved_onsen_data.next_key());
                }
            },
            Message::StageSelect(stage_number) => {
                js::console_log!("to stage {}", stage_number);

                OnsenStatus::show_popup();
                self.stage_number = stage_number;
                self.saved_onsen_data = OnsenStatus::get_onsen_status(self.stage_level, self.stage_number);
            },
            Message::StageClose => {
                if -1 == self.stage_number {
                    return Message::None;
                }

                OnsenStatus::hide_popup();
                self.stage_number = -1;
            },
            Message::StageTweet | Message::StageYoutube => {
                let mut temp_status = OnsenStatus::new(&self.saved_onsen_data.key);
                temp_status.init_from_user_storage();
                temp_status.use_hint_and_save();
                Resource::user_storage().onsen_status.set_onsen_data(&self.saved_onsen_data.key, &temp_status);
                js::console_log!("key = {}", self.saved_onsen_data.key);

                Resource::user_storage().init_onsen_key = self.saved_onsen_data.key.to_string();
                Resource::user_storage().save_data();
            },
            Message::AllComplete => {
                js::console_log!("all complete");
                return Message::ChangeToQuastionPage("complete".to_string());
            },
            _ => (),
        };

        Message::None
    }

    fn rendered(&mut self, _first_render: bool) {
        self.saved_onsen_data.rendered();
    }
}

// 問題出題ページは Topページ も兼ねている
struct QuastionPage {
    sen_op: sen::SenOpManager,
    quastion: &'static OnsenStatus,
    now_status: OnsenStatus,
    cursor_image: Option<HtmlImageElement>,
    first_page: bool,
    opened_door: bool,
}
impl QuastionPage {
    const HINT_PARAM_NAME: &'static str = "hint";
    const POPUP_PARAM_NAME: &'static str = "show_popup";

    fn new(name: &str) -> Self {
        let mut own = Self {
            sen_op: sen::SenOpManager::new( vec![sen::SenOp::Off, sen::SenOp::On, sen::SenOp::Or, sen::SenOp::And, sen::SenOp::Not], Some(0) ),
            quastion: &Resource::onsen_status_manager().onsen_status_list[name],
            now_status: OnsenStatus::new(name),
            cursor_image: None,
            first_page: false,
            opened_door: false,
        };
        own.load_quastion(name);

        if let Some(hint_value) = js::dom::get_param(Self::HINT_PARAM_NAME) {
            if "off" == &hint_value {
                own.now_status.use_hint_and_save();
                
                js::console_log!("hint off.");
                js::popup_alert!("広告によるヒントがまだ使用できません。");
            } else {
                own.now_status.use_hint_and_save();

                js::console_log!("hint on");
            }
            js::dom::redirect(&format!("/index.html?{}=on", Self::POPUP_PARAM_NAME));
        }

        own
    }

    fn new_from_localstorage() -> Self {
        let mut own = Self::new(&Resource::user_storage().init_onsen_key);
        own.first_page = true;

        own
    }

    // 問題の読み込みと現在の状態の初期化
    fn load_quastion(&mut self, name: &str) {
        self.quastion = OnsenStatus::get_onsen_status_from_name(name);
        self.now_status.sen.deep_copy(&self.quastion.sen);
    }

    fn get_content_view(&self, link: &ComponentLink<MainModel>) -> Html {
        html!{
            <div class="container_item_content">
                { self.now_status.get_popup_html(link, false) }
                
                <img id="open_air_bath" src="/resource/image/onsen_ofuro_noone_bg.png" alt="open_air_bath" />
                <div id="wood_kanban_div">
                    <img id="wood_kanban" src="/resource/image/wood_kanban.png" alt="wood_kanban" />
                    <div id="wood_kanban_text_div">
                        <div id="wood_kanban_info">{ &self.quastion.info }</div>
                        <div id="wood_kanban_text">{ self.quastion.temperature }{ "℃" }</div>
                    </div>
                </div>
                <div id="ondokei_div">
                    <img id="ondokei" src="/resource/image/ondokei.png" alt="ondokei" />
                    <div id="ondokei_text_div">
                        <p id="ondokei_text">{ self.now_status.temperature }{ "℃" }</p>
                    </div>
                </div>
                <div id="onsen_mark_frame_div">
                    <img id="onsen_mark_frame" src="/resource/image/onsen_mark_frame.png" alt="onsen_mark_frame" />
                    <div id="onsen_mark_sen">
                    </div>
                </div>
                <img id="tutorial_cursor" src="/resource/image/computer_cursor_finger_white.png" alt="tutorial_cursor" />
            </div>
        }
    }

    /// 温泉マークを移動したときに呼ぶ
    fn moved_onsen_mark(&mut self) -> Message {
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
        let prev_sens = self.now_status.sen.clone();
        let mut new_op = self.now_status.sen.operation(&self.sen_op.get_top(), &mut |(index, _sen)| {
            let sen_image: HtmlImageElement = js::dom::get_element_by_id(&format!("bit_{}", index)).unwrap();
            let rect = sen_image.get_bounding_client_rect();

            // 短角内 は各項目に適用, (On || Off) は全てに適用
            rect.x() <= x && x < rect.x()+rect.width() && rect.y() <= y && y < rect.y()+rect.height()
                || sen_op_top == sen::SenOp::On
                || sen_op_top == sen::SenOp::Off
        });

        // ドラッグ中の温泉マークを削除
        if let Some(image) = self.cursor_image.as_mut() {
            image.remove();
            self.cursor_image = None;
        }

        // sen に変化があるか
        if prev_sens != self.now_status.sen {
            // tutorial がついてると削除して非表示にする
            if let Some(tutorial_cursor) = js::dom::get_element_by_id::<HtmlElement>("tutorial_cursor") {
                tutorial_cursor.class_list().remove_1("tutorial_senop_cursor").unwrap();
                tutorial_cursor.class_list().remove_1("tutorial_move_cursor").unwrap();
            }

            // SenOp::O だったなら消費する
            if self.sen_op.get_top().is_o() {
                self.sen_op.pop();
            }
        }

        // 新しい SenOp があると点滅させる
        if !new_op.is_empty() {
            if let Some(rb_under) = js::dom::get_element_by_id::<HtmlElement>("rb_under") {
                rb_under.class_list().add_1("blink_new_ruby").unwrap();
            }
        }

        let prev_temperature = self.now_status.temperature;
        
        // 各種パラメータの変更: SenOp の追加(あるなら), 操作カウンタ++, 温度再設定, 温泉名へ SenOp の追加
        self.sen_op.append_at_index(&mut new_op);
        self.now_status.ops_count += 1;
        self.now_status.temperature = self.now_status.sen.get_number();
        self.now_status.onsen_name += sen_op_top.to_string().to_lowercase();

        // 温度の上下に応じてSEを流す
        if self.now_status.temperature != self.quastion.temperature {
            if self.now_status.temperature < prev_temperature {
                Resource::sound().play(SoundKind::DownTemp);
            } else if prev_temperature < self.now_status.temperature {
                Resource::sound().play(SoundKind::UpTemp);
            }
        }

        js::console_log!("{:?} / {:?}", self.sen_op, self.now_status);

        // Offの任意 or 正解と同じ温度 なら保存して、結果ポップアップの表示
        if self.sen_op.get_top() != sen::SenOp::Off && self.now_status.temperature != self.quastion.temperature {
            // SenOp::Off 以外 かつ 温度が違うとそのままのページ
            return Message::None;
        }

        self.stage_clear()
    }

    fn stage_clear(&mut self) -> Message {
        if self.sen_op.get_top() != sen::SenOp::Off {
            // ☆
            self.now_status.is_clear = true;
            self.now_status.is_using_onsen |= self.now_status.onsen_name.roma.find("on").is_some();
            self.now_status.is_lower_border |= self.now_status.ops_count <= self.quastion.ops_border;

            // 温泉名の決定
            self.now_status.onsen_name.japanification();
        } else {
            if Resource::user_storage().onsen_status.onsen_status_list[&self.now_status.key].is_cleared() {
                // 過去にクリアしていると初期化しない
                self.now_status = Resource::user_storage().onsen_status.onsen_status_list[&self.now_status.key].clone();
            } else {
                // json から初期化
                self.now_status.init();
                self.now_status.sen.init();
            }
        }

        // クリア時に星の数に応じて「カッポーン」を鳴らす
        let se_kind_count: i8 = self.now_status.is_clear as i8 + self.now_status.is_using_onsen as i8 + self.now_status.is_lower_border as i8;
        match se_kind_count {
            1 => Resource::sound().play(SoundKind::Sen01),
            2 => Resource::sound().play(SoundKind::Sen10),
            3 => Resource::sound().play(SoundKind::Sen11),
            _ => (),
        }

        // 保存
        Resource::user_storage().onsen_status.set_onsen_data(
            &self.now_status.key,
            &self.now_status
        );
        Resource::user_storage().save_data();

        // top なら、結果を表示せずに SelectPage へ
        if "top" == self.now_status.key {
            return Message::ChangeToSelectPage;
        }

        // 結果ポップアップの表示
        OnsenStatus::show_popup();

        Message::None
    }

    fn set_tutorial_animation(&self) {
        if let Some(tutorial_cursor) = js::dom::get_element_by_id::<HtmlElement>("tutorial_cursor") {
            if !tutorial_cursor.class_list().contains("tutorial_senop_cursor") {
                // tutorial.1: op の切り替え
                tutorial_cursor.class_list().remove_1("tutorial_move_cursor").unwrap();
                tutorial_cursor.class_list().add_1("tutorial_senop_cursor").unwrap();
            } else {
                // tutorial.2: 温泉マークへの移動
                tutorial_cursor.class_list().add_1("tutorial_move_cursor").unwrap();
            }
        }
    }

    fn play_bgm(&self) {
        let bgm_kind = match self.now_status.key.as_str() {
            "top" => SoundKind::BgmStageTop,
            _ => SoundKind::BgmStageDefault,
        };

        Resource::sound().set_volume(bgm_kind, 0.5);
        Resource::sound().play_loop(bgm_kind);
    }
}
impl PageTrait for QuastionPage {
    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        if !self.first_page && !self.opened_door {
            link.send_message(Message::OpenDoor);
        }
        
        html! {
            <div class="top_container" id="grand_parent_node">
                <div class="container_item_header">
                    /*{ "banner_area" }*/
                </div>
                <div class="container_item_title">
                    <div id="title_text" class="normal_font">
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
                                    id="rb_under"
                                    data-front-ruby={ self.sen_op.get_front() }
                                    ontouchstart=link.callback(|_| Message::TouchStartFrontSen)
                                    onanimationend=link.callback(|_| Message::BlinkAnimationEnd)
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
                { self.get_content_view(link) }
                <div class="container_item_footer">
                    /*{ "banner_area" }*/
                </div>
                <img id="left_door" src="/resource/image/left_door.png" alt="left_door"
                    ontouchend=link.callback(|event| Message::OpenDoor)
                    onanimationend=link.callback(|event| Message::OpenedDoor) />
                <img id="right_door" src="/resource/image/right_door.png" alt="right_door"
                    ontouchend=link.callback(|event| Message::OpenDoor) />
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
                return self.moved_onsen_mark();
            },
            Message::StageEnter => {
                OnsenStatus::hide_popup();
                
                return Message::ChangeToQuastionPage(
                    self.now_status.key.clone()
                );
            },
            Message::StageBack => {
                return Message::ChangeToSelectPage;
            },
            Message::StageNext => {
                OnsenStatus::hide_popup();

                return Message::ChangeToQuastionPage(
                    self.now_status.next_key()
                );
            },
            Message::TouchStartBackSen => {
                self.sen_op.prev()
            },
            Message::TouchStartFrontSen => {
                self.sen_op.next();
                self.set_tutorial_animation();
        },
            Message::BlinkAnimationEnd => {
                if let Some(rb_under) = js::dom::get_element_by_id::<HtmlElement>("rb_under") {
                    rb_under.class_list().remove_1("blink_new_ruby").unwrap();
                }
            },
            Message::OpenDoor => {
                if self.opened_door {
                    return Message::None;
                }
                self.opened_door = true;
                self.first_page = false;

                let mut is_retry = false;

                if let Some(left_door) = js::dom::get_element_by_id::<HtmlImageElement>("left_door") {
                    if 0 == left_door.class_list().length() {
                        left_door.class_list().add_1("open_left_door").unwrap();
                    } else {
                        is_retry = true;
                    }
                }
                if let Some(right_door) = js::dom::get_element_by_id::<HtmlImageElement>("right_door") {
                    right_door.class_list().add_1("open_right_door").unwrap();
                }

                if is_retry {
                    js::console_log!("retry. opened door");
                    self.play_bgm();
                } else {
                    js::console_log!("open door");
                    Resource::sound().play(SoundKind::StageEnter);
                }
            },
            Message::OpenedDoor => {
                self.play_bgm();
            },
            Message::StageTweet | Message::StageYoutube => {
                Resource::user_storage().init_onsen_key = self.now_status.key.to_string();
                Resource::user_storage().save_data();
                self.now_status.use_hint_and_save();
            },
            _ => (),
        }

        Message::None
    }

    fn rendered(&mut self, first_render: bool) {
        self.now_status.rendered();

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
            if self.first_page && self.now_status.use_hint {
                if let Some(_) = js::dom::get_param(Self::POPUP_PARAM_NAME) {
                    OnsenStatus::show_popup();
                }
            }

            // 初回 かつ 未クリア時 で 最初の方のステージ でのみチュートリアル用のカーソルアニメーションを表示
            if !self.now_status.is_cleared() {
                if "top" == self.now_status.key {
                    self.set_tutorial_animation();
                }
            }
        }
    }
}

struct PageManager {
    page: Box<dyn PageTrait>,
}
impl PageManager {
    fn new() -> Self {
        Self {
            page: Box::new(QuastionPage::new_from_localstorage()),
        }
    }

    fn update(&mut self, message: Message) -> ShouldRender {
        match self.page.as_mut().update(message.clone()) {
            Message::ChangeToQuastionPage(name) => {
                Resource::user_storage().init_onsen_key = name;
                Resource::user_storage().save_data();
                Resource::sound().pause_all();

                self.page = Box::new(QuastionPage::new(&Resource::user_storage().init_onsen_key));
            },
            Message::ChangeToSelectPage => {
                Resource::sound().pause_all();

                self.page = Box::new(SelectPage::new());
            },
            _ => (),
        }

        match message {
            Message::StageBack | Message::StageClose | Message::StageEnter | Message::StageNext
            | Message::StageSelect(_) | Message::TouchStartBackSen | Message::TouchStartFrontSen => {
                Resource::sound().play(SoundKind::ButtonUp);
            },
            _ => (),
        }
        
        true
    }

    fn view(&self, link: &ComponentLink<MainModel>) -> Html {
        html! {
            <div id="base_view">
                { self.page.as_ref().view(link) }
                { Resource::sound().view(link) }
            </div>
        }
    }

    const TEXT_LARGE_LIMIT: usize = 6*3;    // 8文字過多
    // 描画後
    fn rendered(&mut self, first_render: bool) {
        if let Some(stage_name) = js::dom::get_element_by_id::<HtmlElement>("stage_name") {
            if Self::TEXT_LARGE_LIMIT < stage_name.text_content().unwrap().len() {
                // 長すぎると流れるアニメーションを付与する
                stage_name.class_list().add_1("stage_name_flowing").unwrap();
            } else {
                stage_name.class_list().remove_1("stage_name_flowing").unwrap();
            }
        }

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
