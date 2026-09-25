use crate::fishes::fish::Fish;

pub trait Abductable {
    fn can_be_abducted(&self) -> bool;
}

impl Abductable for Fish {
    fn can_be_abducted(&self) -> bool {
        self.species.config().abductable
    }
}
