#![allow(dead_code)]
use bitvec::prelude as bv;

mod game;

const THE_SPELLBOOK: game::Spellbook = game::Spellbook {
    spells: [
        game::SpellRef::Spell1(&game::HarvestScythe{}),
        game::SpellRef::Spell2(&game::ViseGrip{}),
        game::SpellRef::Spell1(&game::HarvestScythe{}),
        game::SpellRef::Spell1(&game::Chisel{}),
        game::SpellRef::Spell1(&game::HarvestScythe{}),
        game::SpellRef::Spell1(&game::DoppelTwice{}),
    ],
    consumed: bv::BitArray::ZERO,
};

fn main() {
    println!("Hello, world!");
}
