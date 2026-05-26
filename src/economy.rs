pub trait Purchasable {
    fn buy_price(&self) -> u32;
    fn display_name(&self) -> &str;
}

pub trait Sellable {
    fn sell_price(&self) -> u32;
    fn display_name(&self) -> &str;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rarity {
    Common,
    Rare,
    Legendary,
}

impl Rarity {
    pub fn catch_weight(self) -> u32 {
        match self {
            Rarity::Common    => 62,
            Rarity::Rare      => 22,
            Rarity::Legendary => 4,
        }
    }

    pub fn fish_buy_price(self) -> u32 {
        match self {
            Rarity::Common    => 126,
            Rarity::Rare      => 341,
            Rarity::Legendary => 7700,
        }
    }
}
