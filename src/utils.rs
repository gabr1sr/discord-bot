use crate::Context;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use serenity::{
    all::{parse_emoji, EmojiIdentifier},
    model::id::UserId,
};
use std::collections::HashMap;
use std::{borrow::Cow, str::FromStr};

pub fn user_ids_from(message: &str) -> Vec<UserId> {
    let re = Regex::new(r"(?P<id>[0-9]+)").unwrap();

    re.captures_iter(message)
        .map(|cap| cap.name("id").unwrap().as_str())
        .map(|id| UserId::from_str(id).unwrap())
        .collect()
}

pub fn emoji_identifiers_from(message: &str) -> Vec<EmojiIdentifier> {
    let re = Regex::new(r"(<a?)?:\w+:([0-9]+>)").unwrap();

    re.find_iter(message)
        .map(|m| m.as_str())
        .map(|e| parse_emoji(e).unwrap())
        .collect()
}

pub fn format_user_ids_list(user_ids: Vec<UserId>) -> String {
    user_ids
        .into_iter()
        .map(|u| format!("- <@{}>", u.get()))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn user_ids_below_user(
    ctx: &Context<'_>,
    user_ids: Vec<UserId>,
    user_id: UserId,
) -> Vec<UserId> {
    let guild = ctx.guild().unwrap();

    user_ids
        .into_iter()
        .filter(|u| guild.greater_member_hierarchy(&ctx, user_id, u).is_some())
        .collect()
}

pub fn reason_into_header(reason: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    // "The X-Audit-Log-Reason header supports 1-512 URL-encoded UTF-8 characters."
    // https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object
    let header_value = match Cow::from(utf8_percent_encode(reason, NON_ALPHANUMERIC)) {
        Cow::Borrowed(value) => String::from(value),
        Cow::Owned(value) => value,
    };

    headers.insert("X-Audit-Log-Reason".to_string(), header_value);
    headers
}

#[test]
fn test_user_ids_from() {
    let expected_ids = vec![1146444622176981142, 1242767296020353056];
    let message = "<@1146444622176981142> 1242767296020353056";
    let ids = user_ids_from(&message);
    assert_eq!(ids, expected_ids);
}

#[test]
fn test_emoji_identifiers_from() {
    let expected_ids = vec![
        EmojiIdentifier::from_str("<:epiccat:1254878341471670292>").unwrap(),
        EmojiIdentifier::from_str("<:omegalul:1073745489775829104>").unwrap(),
        EmojiIdentifier::from_str("<a:kirbypiscando:1242735045140025416>").unwrap(),
    ];

    let message = "<:epiccat:1254878341471670292> <:omegalul:1073745489775829104> <a:kirbypiscando:1242735045140025416>";
    let ids = emoji_identifiers_from(&message);
    assert_eq!(&ids[0].id, &expected_ids[0].id);
    assert_eq!(&ids[1].id, &expected_ids[2].id);
    assert_eq!(&ids[2].id, &expected_ids[3].id);
}
