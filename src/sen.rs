/// Sen の演算の種類
#[derive(Debug)]
pub enum SenOp {
    Off, On, Not, Or, And
}
impl SenOp {
    pub fn operation(&self, x: &mut ux::u2, y: Option<&mut ux::u2>) -> Option<ux::u2> {
        match self {
            Off => {
                *x = ux::u2::MIN;
                None
            },
            On => {
                *x = ux::u2::MAX;
                None
            },
            Not => {
                *x = !*x;
                None
            },
            Or => Some( *x | *y.unwrap() ),
            And => Some( *x & *y.unwrap() ),
        }
    }
}

/// Sen の本数を扱うクラス
#[derive(Debug, PartialEq)]
pub struct Sen {
    pub value: Vec<ux::u2>,
}
