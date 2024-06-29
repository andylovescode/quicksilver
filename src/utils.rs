use serenity::all::{User, UserId};
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use crate::{data::Database, Error};

pub trait GetDB {
	fn db(&self, purpose: &str) -> impl std::future::Future<Output = MutexGuard<Database>> + Send;
}

impl<'a> GetDB for poise::Context<'a, Arc<Mutex<Database>>, Error> {
	async fn db(&self, _: &str) -> MutexGuard<Database> { self.data().lock().await }
}

pub fn calculate_length_to_xp(len: &usize) -> u64 {
	let curve = ((*len as f64) / 15f64).powf(2f64) * 5f64; // curve = (len / 15) ^ 2 * 5

	let max = 15f64;

	let limited = if curve > max { max } else { curve }; // Clamp the curve to 15

	limited.round() as u64
}

pub trait AntiSpamCount {
	fn anti_spam_count(&self) -> usize;
}

impl AntiSpamCount for String {
	fn anti_spam_count(&self) -> usize {
		let mut distinct_chars = vec![];

		for char in self.chars() {
			if char.is_alphabetic() && !distinct_chars.contains(&char) {
				distinct_chars.push(char)
			}
		}

		distinct_chars.len()
	}
}

pub trait Admin {
	fn is_admin(&self) -> bool;
}

impl Admin for User {
	fn is_admin(&self) -> bool { self.id == UserId::new(1136701682131144714) }
}
