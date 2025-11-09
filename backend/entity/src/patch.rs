use sea_orm::{ActiveValue, entity::prelude::*};

impl crate::message::ActiveModel {
    pub fn full_change(&mut self) {
        macro_rules! change {
            ($i:ident) => {
                self.$i = ActiveValue::set(self.$i.take().unwrap());
            };
        }
        change!(inner);
        change!(price);
        change!(token_count);
    }
}
