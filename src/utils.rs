use regex::Regex;
use serenity::model::id::UserId;
use std::str::FromStr;

pub fn user_ids_from(message: &str) -> Vec<UserId> {
    let re = Regex::new(r"(?P<id>[0-9]+)").unwrap();

    re.captures_iter(message)
        .map(|c| c.extract::<1>())
        .map(|(u, _)| UserId::from_str(u).unwrap())
        .collect()
}

#[test]
fn test_user_ids_from() {
    let expected_ids = vec![1146444622176981142, 1242767296020353056];
    let message = "<@1146444622176981142> 1242767296020353056";
    let ids = user_ids_from(&message);
    assert_eq!(ids, expected_ids);
}
