use std::{fmt::Debug, path::Path};

use crate::data::state::SideChannel;
use eyre::{OptionExt, Result};
use state::{DBEvent, DBState};

mod battle;
pub mod items;
pub mod rng;
pub mod state;
pub mod user;

#[derive(Debug)]
pub struct Database {
	state: DBState,
	timeline: Vec<DBEvent>,
	path: Box<Path>,
}

impl Database {
	pub fn new(path: Box<Path>) -> Result<Self> {
		let mut me = Self {
			state: DBState::default(),
			timeline: vec![],
			path,
		};

		let loaded_timeline = if me.path.exists() {
			let file_content = std::fs::read_to_string(&me.path)?;
			serde_json::from_str::<Vec<DBEvent>>(&file_content)?
		} else {
			vec![]
		};

		for entry in &loaded_timeline {
			(me.state, _) = entry.reduce_state(me.state.clone());
		}

		me.timeline = loaded_timeline;

		me.save()?;

		Ok(me)
	}

	pub fn state(&self) -> &DBState { &self.state }

	fn save(&self) -> Result<()> {
		let file_content = serde_json::to_string_pretty(&self.timeline)?;
		std::fs::write(&self.path, file_content)?;

		std::fs::write(
			self.path.to_str().ok_or_eyre("Invalid path")?.to_string() + ".log",
			format!("{:#?}", self.state),
		)?;

		Ok(())
	}

	pub fn add(&mut self, event: DBEvent) -> Result<SideChannel> {
		let (state, side_channel) = event.reduce_state(self.state.clone());

		self.state = state;
		self.timeline.push(event);
		self.save()?;

		Ok(side_channel)
	}
}
