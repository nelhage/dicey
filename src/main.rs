#![allow(dead_code)]
use bitvec::prelude as bv;
use core::array;
use std::{cmp::Ordering, time::Duration};
use std::{collections::BinaryHeap, time::Instant, vec::Vec};

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
    Pass,
    Prep { spell: Die, slot: u8 },
    Cast1 { slot: u8, die: Die },
    Cast2 { slot: u8, d1: Die, d2: Die },
}

fn use_spell(state: &GameState, slot_idx: u8) -> (GameState, SpellRef) {
    let slot = &state.board[slot_idx as usize];
    assert!(slot.uses > 0);
    let spell_idx = slot.spell.unwrap().as_index();
    // assert!(!state.spellbook.consumed[spell_idx]);
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
        Action::Pass => state.clone(),
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

fn all_actions(state: &GameState) -> Vec<Action> {
    let mut actions = Vec::new();
    for slot in 0..4 {
        if state.board[slot].uses > 0 {
            continue;
        }
        for di in 0..6 {
            if state.dice[di] > 0 && !state.spellbook.consumed[di] {
                actions.push(Action::Prep {
                    spell: Die::from_index(di),
                    slot: slot as u8,
                });
            }
        }
        break;
    }

    for si in 0..4 {
        let slot = &state.board[si];
        if slot.spell.is_none() || slot.uses == 0 {
            continue;
        }
        let spell = state.spellbook.spells[slot.spell.unwrap().as_index()];
        match spell {
            SpellRef::Spell1(s) => {
                for di in 0..6 {
                    let die = Die::from_index(di);
                    if state.dice[di] > 0 && s.can_cast(state, die) {
                        actions.push(Action::Cast1 {
                            slot: si as u8,
                            die,
                        })
                    }
                }
            }
            SpellRef::Spell2(s) => {
                for d1 in 0..6 {
                    if state.dice[d1] == 0 {
                        continue;
                    }
                    for d2 in d1..6 {
                        if state.dice[d2] < if d1 == d2 { 2 } else { 1 } {
                            continue;
                        }
                        let d1 = Die::from_index(d1);
                        let d2 = Die::from_index(d2);
                        if s.can_cast(state, d1, d2) {
                            actions.push(Action::Cast2 {
                                slot: si as u8,
                                d1,
                                d2,
                            })
                        }
                    }
                }
            }
        }
    }

    return actions;
}

fn search(state: &GameState, depth: usize) -> (Action, GameState) {
    let mut pv: Action = Action::Pass;
    let mut best = state.clone();

    if depth == 0 {
        return (pv, best);
    }

    for act in all_actions(&state) {
        let child = apply_action(state, act);

        let (_, terminal) = search(&child, depth - 1);

        if terminal.enemy_hp < best.enemy_hp {
            pv = act;
            best = terminal;
        }
    }

    (pv, best)
}

#[derive(Clone)]
struct SearchNode {
    state: GameState,
    depth: usize,
}

impl SearchNode {
    fn key(&self) -> (usize, usize) {
        (self.state.enemy_hp as usize, self.depth)
    }
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
impl Eq for SearchNode {}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key()).reverse()
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const MAX_ELEMS: usize = 10_000_000;
const TICK: Duration = Duration::from_secs(1);

fn best_first() {
    let start = Instant::now();
    let mut tick = start + TICK;
    let mut heap = BinaryHeap::new();
    let initial = SearchNode {
        state: initial_state(),
        depth: 0,
    };
    let mut best = initial.clone();
    heap.push(initial);
    for i in 0.. {
        let nd = heap.pop().unwrap();
        if nd > best {
            best = nd.clone();
        }
        for act in all_actions(&nd.state) {
            heap.push(SearchNode {
                state: apply_action(&nd.state, act),
                depth: nd.depth + 1,
            });
        }
        if i % 10_000 == 0 {
            let now = Instant::now();
            if now >= tick {
                tick += TICK;
                let elapsed = now.duration_since(start);

                println!(
                    "i={} n={} t={:?} depth={} best={:?}",
                    i,
                    heap.len(),
                    elapsed,
                    best.depth,
                    best.state,
                );
            }
        }
        if heap.len() > MAX_ELEMS {
            break;
        }
    }
}

fn main() {
    /*
    let st = initial_state();
    println!("Game starts: {:?}", st);
    let s1 = apply_action(
        &st,
        Action::Prep {
            spell: Die::from_pips(1),
            slot: 0,
        },
    );
    println!("Prep 1 into 10: {:?}", s1);
    let s2 = apply_action(
        &s1,
        Action::Cast1 {
            slot: 0,
            die: Die::from_pips(2),
        },
    );
    println!("Cast 0 with 2: {:?}", s2);
    */

    /*
    for depth in 0.. {
        let init = initial_state();
        let (pv, terminal) = search(&init, depth);
        println!("depth={}", depth);
        println!("pv: {:?}", pv);
        println!("terminal: {:?}", terminal);
    }
    */
    best_first();
}
