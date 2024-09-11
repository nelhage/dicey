#![allow(dead_code)]
use bitvec::prelude as bv;

mod game;
use game::*;

const THE_SPELLBOOK: Spellbook = Spellbook {
    spells: [
        SpellRef::Spell1(&HarvestScythe{}),
        SpellRef::Spell2(&ViseGrip{}),
        SpellRef::Spell1(&HarvestScythe{}),
        SpellRef::Spell1(&Chisel{}),
        SpellRef::Spell1(&HarvestScythe{}),
        SpellRef::Spell1(&DoppelTwice{}),
    ],
    consumed: bv::BitArray::ZERO,
};

#[derive(Debug, Copy, Clone)]
enum Action {
    Prep{spell: Die, slot: u8},
    Cast1{slot: u8, die: Die},
    Cast2{slot: u8, d1: Die, d2: Die},
}

fn apply_action(state: &GameState, action: Action) -> GameState {
    match action {
        Action::Prep{spell, slot} => {
            assert!(!state.spellbook.consumed[spell.as_index()]);
            let mut board = state.board.clone();
            board[slot as usize] = SpellSlot{
                spell: Some(spell),
                uses: state.spellbook.spells[spell.as_index()].initial_uses(),
            };
            return GameState {
                board,
                ..state.use_die(spell)
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
