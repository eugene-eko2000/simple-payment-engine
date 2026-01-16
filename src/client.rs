use rust_decimal::Decimal;

pub struct Client {
    pub id: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Client {
            id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }
}
