use regex::Regex;
use serenity::model::id::UserId;

pub fn user_ids_from(message: &str) -> Vec<UserId> {
    let raw_user_ids: Vec<u64> = raw_ids_from(message);
    let mut user_ids: Vec<UserId> = Vec::new();

    for id in raw_user_ids.iter() {
        user_ids.push(UserId::new(*id));
    }

    user_ids
}

fn raw_ids_from(message: &str) -> Vec<u64> {
    let re = Regex::new(r"(?P<id>[0-9]+)").unwrap();
    let mut ids: Vec<u64> = Vec::new();

    for (_, [id]) in re.captures_iter(message).map(|c| c.extract()) {
        ids.push(id.parse::<u64>().unwrap());
    }

    ids
}
