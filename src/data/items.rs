use serde::{Deserialize, Serialize};
use serenity::all::{Colour};

#[derive(Debug)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythical,
    Unique
}

impl Rarity {
    pub fn color(&self) -> Colour {
        match self {
            Rarity::Common => Colour(0xFFFFFF),
            Rarity::Uncommon => Colour(0x00FFA8),
            Rarity::Rare => Colour(0xF4DC00),
            Rarity::Mythical => Colour(0x6A00DF),
            Rarity::Unique => Colour(0xFF0081)
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Rarity::Common => "[common]",
            Rarity::Uncommon => "[uncommon]",
            Rarity::Rare => "[rare]",
            Rarity::Mythical => "[mythical]",
            Rarity::Unique => "[unique]"
        }
    }
}

pub struct ItemInfo {
    pub name: String,
    pub rarity: Rarity,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, poise::ChoiceParameter)]
pub enum InventoryItem {
    #[name = "Testing Gizmo"]
    TestGizmo,

    #[name = "Stick"]
    Stick,

    #[name = "Rock"]
    Rock,

    #[name = "Wand"]
    Wand,

    #[name = "Scythe (Vivi)"]
    ScytheVivi,

    #[name = "Ace of Spades"]
    Ace,

    #[name = "Cross (Minsley)"]
    CrossMinsley,

    #[name = "The Oracle Amulet"]
    OracleAmulet,

    #[name = "Gun"]
    Gun
}

impl InventoryItem {
    pub fn info(&self) -> ItemInfo {
        match self {
            InventoryItem::TestGizmo => ItemInfo {
                    name: "Testing Gizmo".to_string(),
                    rarity: Rarity::Mythical,
                    description: "An oozing formless blob of purple and black. No matter what angle you look at, the checkerboard is in the same position in your eyes.".to_string(),
            },
            InventoryItem::Stick => ItemInfo {
                name: "Stick".to_string(),
                rarity: Rarity::Common,
                description: "A wooden rod from a tree.".to_string()
            },
            InventoryItem::Rock => ItemInfo {
                name: "Rock".to_string(),
                rarity: Rarity::Common,
                description: "A stone-cold... stone, comes from the ground.".to_string()
            },
            InventoryItem::Wand => ItemInfo {
                name: "Wand".to_string(),
                rarity: Rarity::Rare,
                description: "A mystical wand imbued with magic, capable of casting ancient spells.".to_string()
            },
            InventoryItem::ScytheVivi => ItemInfo {
                name: "Vivi's Scythe".to_string(),
                rarity: Rarity::Unique,
                description: "Vivi's magical yet deadly scythe.".to_string()
            },
            InventoryItem::Ace => ItemInfo {
                name: "Ace of Spades".to_string(),
                rarity: Rarity::Mythical,
                description: "The most powerful item, can only be used once, but could change the course of a fight.".to_string()
            },
            InventoryItem::CrossMinsley => ItemInfo {
                name: "Minsley's Cross".to_string(),
                rarity: Rarity::Unique,
                description: "A dangerous weapons for rapid hits and stuns.".to_string()
            },
            InventoryItem::OracleAmulet => ItemInfo {
                name: "The Oracle's Amulet".to_string(),
                rarity: Rarity::Unique,
                description: "A rune with an eye in the middle. It is said to contain The Oracle.".to_string()
            },
            InventoryItem::Gun => ItemInfo {
                name: "Gun".to_string(),
                rarity: Rarity::Uncommon,
                description: "How the hell is this legal? (deals a lot of damage, but large surface area)".to_string()
            }
        }
    }
}