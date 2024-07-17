use poise::{ChoiceParameter, CommandParameterChoice};

pub static PLACES: &[Place] = &[
	Place::Forest,
	Place::Capital,
	Place::HomeMinsley,
	Place::HomeZyex,
	Place::HomeVivi,
	Place::DevTest,
];

#[derive(Copy, Clone)]
pub enum Place {
	// Basic Environments
	Forest,

	// Key lore areas
	Capital,

	// Player homes
	HomeMinsley,
	HomeZyex,
	HomeMoonpool,
	HomeVivi,

	// Development areas
	DevTest,
}

impl Place {
	pub fn name(&self) -> &'static str {
		match self {
			Place::Forest => "The Forest",
			Place::Capital => "The Capital",
			Place::HomeMinsley => "Minsley Manor",
			Place::HomeZyex => "Zyex's Home",
			Place::HomeMoonpool => "Moonpool Manor",
			Place::HomeVivi => "Vivi's Tower?",
			Place::DevTest => "Development Zone",
		}
	}

	pub fn id(&self) -> String {
		self.name()
			.to_lowercase()
			.chars()
			.map(|x| {
				if x.is_alphanumeric() {
					x.to_string()
				} else {
					" ".to_string()
				}
			})
			.collect::<Vec<_>>()
			.join("")
			.split(" ")
			.map(|x| x.trim())
			.filter(|x| !x.is_empty())
			.collect::<Vec<_>>()
			.join("-")
	}
}

impl ChoiceParameter for Place {
	fn list() -> Vec<CommandParameterChoice> {
		PLACES
			.iter()
			.map(|x| CommandParameterChoice {
				name: x.name().to_string(),
				localizations: Default::default(),
				__non_exhaustive: (),
			})
			.collect()
	}

	fn from_index(index: usize) -> Option<Self> { PLACES.get(index).copied() }

	fn from_name(name: &str) -> Option<Self> { PLACES.iter().find(|x| x.name() == name).copied() }

	fn name(&self) -> &'static str { self.name() }

	fn localized_name(&self, _: &str) -> Option<&'static str> { None }
}
