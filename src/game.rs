use std::fmt;
use std::num::NonZeroU8;

use bitvec::prelude as bv;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Die(NonZeroU8);

impl Die {
    pub fn as_index(self) -> usize {
        (self.0.get() - 1) as usize
    }
    pub fn pips(self) -> u8 {
        self.0.get() as u8
    }
    pub fn from_pips(pips: u8) -> Self {
        if pips < 1 || pips > 6 {
            panic!("bad pips: {}", pips);
        }
        Die((pips).try_into().unwrap())
    }
    pub fn from_index(index: usize) -> Self {
        if index > 5 {
            panic!("bad index: {}", index);
        }
        Die(((index + 1) as u8).try_into().unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub dice: [u8; 6],
    pub spellbook: Spellbook,
    pub board: [SpellSlot; 4],
    pub enemy_hp: u8,
}

impl GameState {
    pub fn use_die(&self, die: Die) -> GameState {
        let mut dice = self.dice;
        assert!(dice[die.as_index()] > 0);
        dice[die.as_index()] -= 1;
        return GameState {
            dice,
            ..self.clone()
        };
    }
}

#[derive(Clone, Debug)]
pub struct Spellbook {
    pub spells: [SpellRef; 6],
    pub consumed: bv::BitArray<u8>,
}

#[derive(Copy, Clone)]
pub enum SpellRef {
    Spell1(&'static dyn Spell1),
    Spell2(&'static dyn Spell2),
}

impl SpellRef {
    pub fn spell_name(&self) -> &'static str {
        match *self {
            SpellRef::Spell1(s1) => s1.name(),
            SpellRef::Spell2(s2) => s2.name(),
        }
    }
    pub fn initial_uses(&self) -> u8 {
        match *self {
            SpellRef::Spell1(s1) => s1.initial_uses(),
            SpellRef::Spell2(s2) => s2.initial_uses(),
        }
    }
}

impl fmt::Debug for SpellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Spell")
            .field("name", &self.spell_name())
            .finish()
    }
}

impl Spellbook {
    fn consume(&self, die: Die) -> Option<Self> {
        let mut consumed = self.consumed;
        if consumed[die.as_index()] {
            return None;
        }
        consumed.set(die.as_index(), true);
        return Some(Spellbook { consumed, ..*self });
    }
}

// Helper struct to format spell names
struct SpellNames([&'static dyn BaseSpell; 6]);

impl fmt::Debug for SpellNames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|spell| spell.name()))
            .finish()
    }
}
#[derive(Debug, Clone)]
pub struct SpellSlot {
    pub spell: Option<Die>,
    pub uses: u8,
}

pub trait BaseSpell {
    fn name(&self) -> &str;
    fn initial_uses(&self) -> u8 {
        1
    }
}

pub trait Spell1: BaseSpell {
    fn can_cast(&self, _state: &GameState, _die: Die) -> bool {
        true
    }
    fn cast_spell(&self, state: &GameState, die: Die) -> GameState;
}

pub trait Spell2: BaseSpell {
    fn can_cast(&self, _state: &GameState, _d1: Die, _d2: Die) -> bool {
        true
    }
    fn cast_spell(&self, state: &GameState, d1: Die, d2: Die) -> GameState;
}

pub struct HarvestScythe;
impl BaseSpell for HarvestScythe {
    fn name(&self) -> &str {
        "Harvest Scythe"
    }
}
impl Spell1 for HarvestScythe {
    fn can_cast(&self, state: &GameState, die: Die) -> bool {
        !state.spellbook.consumed[die.as_index()]
    }

    fn cast_spell(&self, state: &GameState, die: Die) -> GameState {
        GameState {
            spellbook: state.spellbook.consume(die).unwrap(),
            enemy_hp: state.enemy_hp - (5 * die.pips()) as u8,
            ..state.clone()
        }
    }
}

pub struct ViseGrip;
impl BaseSpell for ViseGrip {
    fn name(&self) -> &str {
        "Vise Grip"
    }
}

impl Spell2 for ViseGrip {
    fn cast_spell(&self, state: &GameState, d1: Die, d2: Die) -> GameState {
        if d1 == d2 {
            return state.clone();
        }
        let mut dice = state.dice;
        let delta = if d1 > d2 {
            d1.pips() - d2.pips()
        } else {
            d2.pips() - d1.pips()
        } as u8;
        dice[delta as usize] += 3;
        GameState {
            dice,
            ..state.clone()
        }
    }
}

pub struct Chisel;
impl BaseSpell for Chisel {
    fn name(&self) -> &str {
        "Chisel"
    }
    fn initial_uses(&self) -> u8 {
        2
    }
}

impl Spell1 for Chisel {
    fn can_cast(&self, _state: &GameState, die: Die) -> bool {
        die.pips() > 1
    }

    fn cast_spell(&self, state: &GameState, die: Die) -> GameState {
        let mut dice = state.dice.clone();
        dice[die.as_index() - 1] += 1;
        dice[Die::from_pips(1).as_index()] += 1;
        GameState {
            dice,
            ..state.clone()
        }
    }
}

pub struct DoppelTwice;
impl BaseSpell for DoppelTwice {
    fn name(&self) -> &str {
        "DoppelTwice"
    }
    fn initial_uses(&self) -> u8 {
        2
    }
}
impl Spell1 for DoppelTwice {
    fn can_cast(&self, _state: &GameState, die: Die) -> bool {
        die.pips() <= 5
    }
    fn cast_spell(&self, state: &GameState, die: Die) -> GameState {
        let mut dice = state.dice.clone();
        let mut double = die.pips() * 2;
        if double > 6 {
            dice[Die::from_pips(double - 6).as_index()] += 1;
            double -= 6;
        }
        dice[Die::from_pips(double).as_index()] += 1;
        GameState {
            dice,
            ..state.clone()
        }
    }
}
