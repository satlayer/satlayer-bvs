use cw_storage_plus::Item;

pub(crate) const PAUSED: Item<bool> = Item::new("paused");
