use std::str::FromStr;

use regex::Regex;
use serenity::model::id::UserId;

pub fn user_ids_from(message: &str) -> Vec<UserId> {
    let re = Regex::new(r"(?P<id>[0-9]+)").unwrap();

    re.captures_iter(message)
        .map(|c| c.extract::<1>())
        .map(|(u, _)| UserId::from_str(u).unwrap())
        .collect()
}
