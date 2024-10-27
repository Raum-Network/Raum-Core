use soroban_sdk::{Symbol, Address, Env};

pub struct PairTokenEvents;

impl PairTokenEvents {
    pub fn initialized(e: &Env, admin: &Address) {
        let topics = (Symbol::new(e, "initialized"), admin);
        e.events().publish(topics, ());
    }

    pub fn transfer(e: &Env, from: &Address, to: &Address, amount: &i128) {
        let topics = (Symbol::new(e, "transfer"), from, to);
        e.events().publish(topics, *amount);
    }

    pub fn approval(e: &Env, owner: &Address, spender: &Address, amount: &i128) {
        let topics = (Symbol::new(e, "approval"), owner, spender);
        e.events().publish(topics, *amount);
    }

    pub fn mint(e: &Env, to: &Address, amount: &i128) {
        let topics = (Symbol::new(e, "mint"), to);
        e.events().publish(topics, *amount);
    }

    pub fn burn(e: &Env, from: &Address, amount: &i128) {
        let topics = (Symbol::new(e, "burn"), from);
        e.events().publish(topics, *amount);
    }
}