use serde::{
    Deserialize,
    Serialize,
};

/// Sen の演算の種類
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SenOp {
    Off, On, Not, Or, And, OrO(Sen), AndO(Sen),
}
impl SenOp {
    pub const OFF: u8 = 0b00;
    pub const ON: u8 = 0b10;

    /// x に対して self の演算を行う, もし戻り値があれば返す
    pub fn operation(&self, sen: &mut Sen) -> Option<Self> {
        match self {
            Self::Off => {
                sen.bit = Self::OFF;
                None
            },
            Self::On => {
                sen.bit = Self::ON;
                None
            },
            Self::Not => {
                sen.bit = (!sen.bit) & 0b00000011;
                None
            },
            Self::OrO(out_sen) | Self::AndO(out_sen) => {
                sen.bit = out_sen.bit;
                None
            },
            Self::Or => Some( Self::OrO( Sen::new(sen.bit | Self::ON) ) ),
            Self::And => Some( Self::AndO( Sen::new(sen.bit & Self::ON) ) ),
        }
    }

    /// self が {出力} かどうか
    pub fn is_o(&self) -> bool {
        match self {
            Self::OrO(_) | Self::AndO(_) => true,
            _ => false,
        }
    }

    pub fn to_file_name(&self) -> String {
        match self {
            Self::OrO(sen) | Self::AndO(sen) => {
                let self_string = self.to_string();
                let self_string = self_string.split_at(self_string.len()-1).0;
                match sen.bit {
                    0 => "/resource/image//mark_offsen.png".to_string(),
                    2 => format!( "/resource/image//mark_{}sen.png", self_string ),
                    _ => format!( "/resource/image//mark_{}sen_0b{:02b}.png", self_string, sen.bit ),
                }
            }.to_lowercase(),
            _ => format!( "/resource/image//mark_{}sen.png", self ).to_lowercase(),
        }
    }

    fn to_string(&self) -> String {
        format!("{}", *self)
    }
}
impl std::fmt::Display for SenOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OrO(_) => write!(f, "OrO"),
            Self::AndO(_) => write!(f, "AndO"),
            _ => write!(f, "{:?}", *self),
        }
    }
}

#[derive(Debug)]
pub struct SenOpManager {
    sen_op_list: Vec<SenOp>,
    pub sen_index: usize,
}
impl SenOpManager {
    pub fn new(sen_op_list: Vec<SenOp>, default_sen_index: Option<usize>) -> Self {
        Self {
            sen_op_list,
            sen_index: default_sen_index.unwrap_or(0),
        }
    }

    /// sen_index の一つ前の SenOp を返す
    pub fn get_back(&self) -> SenOp {
        self.sen_op_list[ (self.sen_index + (self.sen_op_list.len()-1)) % self.sen_op_list.len() ]
    }
    /// sen_index の SenOp を返す
    pub fn get_top(&self) -> SenOp {
        self.sen_op_list[ self.sen_index ]
    }
    /// sen_index の一つ次の SenOp を返す
    pub fn get_front(&self) -> SenOp {
        self.sen_op_list[ (self.sen_index + 1) % self.sen_op_list.len() ]
    }

    /// sen_index を一つ戻す(循環)
    pub fn prev(&mut self) {
        self.sen_index = (self.sen_index + (self.sen_op_list.len()-1)) % self.sen_op_list.len()
    }
    /// sen_index を一つ進める(循環)
    pub fn next(&mut self) {
        self.sen_index = (self.sen_index + 1) % self.sen_op_list.len()
    }

    /// SenOp のリストに sen_index の位置から new_op_list を加える
    pub fn append_at_index(&mut self, new_op_list: &mut Vec<SenOp>) {
        let sen_index_next = self.sen_index + 1;
        self.sen_op_list.splice( sen_index_next..sen_index_next, new_op_list.iter().cloned() );
    }
    /// SenOp のリストから index を削除する
    pub fn remove(&mut self, index: usize) {
        self.sen_op_list.remove(index);
    }
    /// SenOp のリストから sen_index を削除する
    pub fn pop(&mut self) -> SenOp {
        let ret = self.sen_op_list[self.sen_index];
        self.remove(self.sen_index);
        if self.sen_op_list.len() <= self.sen_index {
            self.sen_index -= 1;
        }

        ret
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Sen {
    pub bit: u8
}
impl Sen {
    pub fn new(bit: u8) -> Self {
        Self {
            bit,
        }
    }

    pub fn to_file_name(&self, index: usize) -> String {
        format!("/resource/image/mark_{}sen_0b{:02b}.png", if index % 2 == 0 { "small_" } else { "" }, self.bit)
    }
}

/// Sen の本数を扱うクラス
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SenManager {
    sen_list: Vec<Sen>,
}
impl SenManager {
    pub fn init(&mut self) {
        self.operation(&SenOp::Off, &mut |(_index, _sen)|{ true });
    }

    pub fn deep_copy(&mut self, x: &Self) {
        self.sen_list = x.sen_list.clone();
        self.sen_list = self.sen_list.iter_mut().map(|sen| { (*sen).bit = 0; *sen } ).collect();
    }

    /// 現在の sen_list を 2 進数へ変換して、つなげて i32 として返す
    pub fn get_number(&self) -> i32 {
        let binary_string = self.sen_list.as_slice().iter().map(|sen| format!("{:02b}", (*sen).bit) );

        isize::from_str_radix(&binary_string.collect::<Vec<String>>().join(""), 2).unwrap() as i32
    }

    /// sen_list を for_each で回す
    pub fn for_each<F>(&self, f: F)
    where
        Self: Sized,
        F: FnMut( (usize, &Sen) ),
    {
        self.sen_list.iter().enumerate().for_each(f);
    }

    /// sen_list の各要素に対して sen_op.operation を当てる
    /// 出力があるとまとめて返す
    pub fn operation<F>(&mut self, sen_op: &SenOp, f: &mut F) -> Vec<SenOp>
    where
        Self: Sized,
        F: FnMut( (usize, &mut Sen) ) -> bool
    {
        let mut new_op: Vec<SenOp> = Vec::new();

        self.sen_list = self.sen_list.iter_mut().enumerate().map(|(index, sen)| {
            if f( (index, sen) ) {
                if let Some(out_op) = sen_op.operation(sen) {
                    // 出力がある演算だと op を追加する
                    new_op.push(out_op);
                }
            }

            *sen
        }).collect();

        new_op
    }
}
