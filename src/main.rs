#![allow(dead_code)]
use bitvec::prelude as bv;
use core::array;

mod game;
use game::*;

const THE_SPELLBOOK: Spellbook = Spellbook {
    spells: [
        SpellRef::Spell1(&HarvestScythe {}),
        SpellRef::Spell2(&ViseGrip {}),
        SpellRef::Spell1(&HarvestScythe {}),
        SpellRef::Spell1(&Chisel {}),
        SpellRef::Spell1(&HarvestScythe {}),
        SpellRef::Spell1(&DoppelTwice {}),
    ],
    consumed: bv::BitArray::ZERO,
};

fn initial_state() -> GameState {
    GameState {
        dice: [1, 1, 1, 1, 1, 1],
        spellbook: THE_SPELLBOOK,
        board: array::from_fn(|_| SpellSlot {
            spell: None,
            uses: 0,
        }),
        enemy_hp: 105,
    }
}

#[derive(Debug, Copy, Clone)]
enum Action {
    Prep { spell: Die, slot: u8 },
    Cast1 { slot: u8, die: Die },
    Cast2 { slot: u8, d1: Die, d2: Die },
}

fn use_spell(state: &GameState, slot_idx: u8) -> (GameState, SpellRef) {
    let slot = &state.board[slot_idx as usize];
    assert!(slot.uses > 0);
    let spell_idx = slot.spell.unwrap().as_index();
    assert!(!state.spellbook.consumed[spell_idx]);
    let spell = state.spellbook.spells[spell_idx];

    let mut new_board = state.board.clone();
    new_board[slot_idx as usize].uses -= 1;
    (
        GameState {
            board: new_board,
            ..state.clone()
        },
        spell,
    )
}

fn apply_action(state: &GameState, action: Action) -> GameState {
    match action {
        Action::Prep { spell, slot } => {
            assert!(!state.spellbook.consumed[spell.as_index()]);
            let mut board = state.board.clone();
            board[slot as usize] = SpellSlot {
                spell: Some(spell),
                uses: state.spellbook.spells[spell.as_index()].initial_uses(),
            };
            return GameState {
                board,
                ..state.use_die(spell)
            };
        }
        Action::Cast1 { slot, die } => {
            let (state, spell) = use_spell(state, slot);
            match spell {
                SpellRef::Spell1(s) => s.cast_spell(&state.use_die(die), die),
                SpellRef::Spell2(_) => {
                    panic!("cast1 on a spell2")
                }
            }
        }
        Action::Cast2 { slot, d1, d2 } => {
            let (state, spell) = use_spell(state, slot);
            match spell {
                SpellRef::Spell1(_) => {
                    panic!("cast2 on a spell1")
                }
                SpellRef::Spell2(s) => s.cast_spell(&state.use_die(d1).use_die(d2), d1, d2),
            }
        }
    }
}

fn main() {
    let st = initial_state();
    println!("Game starts: {:?}", st);
    let s1 = apply_action(
        &st,
        Action::Prep {
            spell: Die::with_pips(1),
            slot: 0,
        },
    );
    println!("Prep 1 into 10: {:?}", s1);
    let s2 = apply_action(
        &s1,
        Action::Cast1 {
            slot: 0,
            die: Die::with_pips(2),
        },
    );
    println!("Cast 0 with 2: {:?}", s2);
}
