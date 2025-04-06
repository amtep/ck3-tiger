use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::pdxfile::PdxEncoding;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct CountryTag {
    file: Token,
}

inventory::submit! {
    ItemLoader::Full(GameFlags::Hoi4, Item::CountryTag, PdxEncoding::Utf8NoBom, ".txt", LoadAsFile::Yes, Recursive::No, CountryTag::add)
}

impl CountryTag {
    pub fn add(db: &mut Db, _file: Token, mut block: Block) {
        for (key, value) in block.drain_assignments_warn() {
            let fake_block = Block::new(key.loc);
            db.add(Item::CountryTag, key, fake_block, Box::new(Self { file: value }));
        }
    }
}

impl DbKind for CountryTag {
    fn validate(&self, _key: &Token, _block: &Block, data: &Everything) {
        let pathname = format!("common/{}", &self.file);
        data.verify_exists_implied(Item::File, &pathname, &self.file);
    }
}
