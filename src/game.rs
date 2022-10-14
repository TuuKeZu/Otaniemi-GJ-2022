use crate::messages::WsMessage;
use crate::packets::*;
use actix::prelude::Recipient;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::time::SystemTime;
use uuid::Uuid;

// https://www.unorules.org/wp-content/uploads/2021/03/All-Uno-cards-how-many-cards-in-uno.png

type Socket = Recipient<WsMessage>;

#[derive(Debug)]
pub struct Game {
    pub id: Uuid,
    pub active: bool,
    pub players: HashMap<Uuid, Player>
}

impl Default for GameStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Game {
        Game {
            id: Uuid::new_v4(),
            active: false,
            players: HashMap::new(),
        }
    }

    pub fn leave(&mut self, id: Uuid) {
        if self.players.contains_key(&id) {
            self.players.remove(&id);
        } else {
            return;
        }

        if self.active {
            self.broadcast(&to_json(PacketType::Message(
                "Server".to_string(),
                "Game ended due to one of the players leaving".to_string(),
            )));
            self.end();
        }
    }

    pub fn get_player(&mut self, id: &Uuid) -> &Player {
        self.players.get(id).unwrap()
    }

    fn send_message(&self, message: &str, id: &Uuid) {
        if let Some(socket_recipient) = self.players.get(id) {
            let _ = socket_recipient
                .socket
                .do_send(WsMessage(message.to_owned()));
        } else {
            println!("Couldn't find anyone to send message to");
        }
    }

    pub fn emit(&self, id: &Uuid, data: &str) {
        self.send_message(data, id);
    }

    pub fn broadcast(&self, data: &str) {
        for id in self.players.keys() {
            self.send_message(data, id);
        }
    }

    pub fn broadcast_ignore_self(&self, self_id: Uuid, data: &str) {
        for id in self.players.keys() {
            if &self_id != id {
                self.send_message(data, id);
            }
        }
    }

    pub fn init_player(&mut self, id: &Uuid, username: &str) {
        let host = self.players.len() == 1;
        let p: Option<&mut Player> = self.players.get_mut(id);

        if let Some(p) = p {
            p.username = String::from(username);
            p.is_connected = true;
            p.is_host = host;
        }

        if host {
            self.emit(
                id,
                &to_json(PacketType::Message(
                    "Server".to_string(),
                    "You are the host".to_string(),
                )),
            )
        }
    }

    pub fn start(&mut self) {
        /* 
        let deck = &mut self.deck;
        self.placed_deck
            .push_front(Card::get_allowed_start_card(deck));

        self.active = true;

        for id in self.players.keys_mut() {
            self.draw_cards(8, id);
            self.update_card_status(&id);
        }

        self.give_turn();
        self.statistics.game_started();

        self.broadcast(&to_json(PacketType::Message(
            "Server".to_string(),
            "The host has started the game".to_string(),
        )));
        */
    }

    pub fn end(&mut self) {
        println!("Player won the game");
        /* 
        self.statistics.game_ended();
        self.statistics.player_count = self.players.len();
        let mut placements = self.players.sort_by_cards();
        let winner = placements.pop_front().unwrap();

        let p = PacketType::WinUpdate(
            winner.id,
            winner.username.clone(),
            placements.iter().map(|p| p.username.clone()).collect(),
            self.statistics.clone(),
        );

        self.broadcast(&to_json(p));

        self.active = false;
        */
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: Uuid,
    pub socket: Socket,
    pub username: String,
    pub is_connected: bool,
    pub is_host: bool,
    pub cards: Vec<Card>,
    pub waiting: bool,
    actions: Vec<Actions>,
}

impl Player {
    pub fn new(id: Uuid, socket: &Socket) -> Player {
        Player {
            id,
            socket: socket.to_owned(),
            username: String::from("connecting..."),
            is_host: false,
            is_connected: false,
            cards: Vec::new(),
            waiting: false,
            actions: Vec::new(),
        }
    }

    pub fn can_end(&self) -> bool {
        // Player can end their turn only if they have placed one card or drawn 3 cards
        self.actions
            .iter()
            .filter(|a| **a == Actions::PlaceCard)
            .count()
            >= 1
            || self
                .actions
                .iter()
                .filter(|a| **a == Actions::DrawCard)
                .count()
                >= 3
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Actions {
    DrawCard,
    PlaceCard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub r#type: Type,
    pub color: Color,
    pub owner: Option<Uuid>,
}

#[derive(strum_macros::Display, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Color {
    Red,
    Blue,
    Green,
    Yellow,
}

impl Color {
    pub fn iter() -> Vec<Color> {
        vec![Color::Red, Color::Blue, Color::Green, Color::Yellow]
    }
}

#[derive(strum_macros::Display, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Type {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Block,
    Reverse,
    DrawTwo,
    Switch,
    DrawFour,
}

impl Type {
    pub fn iter() -> Vec<Type> {
        vec![
            Type::Zero,
            Type::One,
            Type::Two,
            Type::Three,
            Type::Four,
            Type::Five,
            Type::Six,
            Type::Seven,
            Type::Eight,
            Type::Nine,
            Type::Block,
            Type::Reverse,
            Type::DrawTwo,
            Type::Switch,
            Type::DrawFour,
        ]
    }
}

impl Card {
    fn new(r#type: Type, color: Color) -> Card {
        Card {
            r#type,
            color,
            owner: None,
        }
    }

    fn new_with_owner(r#type: Type, color: Color, owner: Option<Uuid>) -> Card {
        Card {
            r#type,
            color,
            owner,
        }
    }

    fn generate_deck() -> VecDeque<Card> {
        let mut l: Vec<Card> = Vec::new();

        for c in &Color::iter() {
            for t in &Type::iter() {
                l.push(Card::new(t.clone(), c.clone()));
                l.push(Card::new(t.clone(), c.clone()));
            }
        }
        l.shuffle(&mut thread_rng());
        VecDeque::from(l)
    }

    fn get_allowed_start_card(deck: &VecDeque<Card>) -> Card {
        let disallowed_types = vec![
            Type::Block,
            Type::Switch,
            Type::DrawFour,
            Type::Reverse,
            Type::DrawTwo,
        ];
        /*
        deck.iter()
            .filter(|card| !disallowed_types.contains(&card.r#type))
            .collect::<VecDeque<&Card>>()
            .pop_back()
            .unwrap()
            .clone()
        */
        Card::new(Type::Five, Color::Red)
    }

    fn get_allowed_cards(last_card: Card, deck: Vec<Card>, owner: Uuid) -> Vec<Card> {
        let mut l = Vec::new();
        let special = [Type::Switch, Type::DrawFour];
        let draw_cards = [Type::DrawTwo, Type::DrawFour];

        for card in deck {
            if last_card.owner == Some(owner) && last_card.owner.is_some() {
                // SAME TYPES
                if card.r#type == last_card.r#type {
                    l.push(card);
                    continue;
                }
            } else {
                if last_card.owner.is_none() {
                    // SPECIAL CARDS
                    if special.contains(&card.r#type) {
                        l.push(card);
                        continue;
                    }

                    // SAME COLORED CARDS
                    if card.color == last_card.color {
                        l.push(card);
                        continue;
                    }

                    // SAME TYPES
                    if card.r#type == last_card.r#type {
                        l.push(card);
                        continue;
                    }
                } else if draw_cards.contains(&last_card.r#type) && last_card.owner != Some(owner) {
                    // SPECIAL CARDS
                    if last_card.r#type == card.r#type {
                        l.push(card);
                        continue;
                    }
                } else if last_card.owner != Some(owner) {
                    // SPECIAL CARDS
                    if special.contains(&card.r#type) {
                        l.push(card);
                        continue;
                    }

                    // SAME COLORED CARDS
                    if card.color == last_card.color {
                        l.push(card);
                        continue;
                    }

                    // SAME TYPES
                    if card.r#type == last_card.r#type {
                        l.push(card);
                        continue;
                    }
                }
            }
        }

        l
    }
}

pub fn to_json(data: PacketType) -> String {
    serde_json::to_string(&data).unwrap()
}
