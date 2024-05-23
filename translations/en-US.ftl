# Commands
ping = ping
     .description = Bot replies with Pong!

help = help
     .description = Command helper
     .command = command
     .command-description = Command name

database = database
     .description = Execute a testing query in the database

infractions = infractions
     .description = Infractions
     .add = add
     .add-description = Add new infraction to the table
     .add-id = id
     .add-id-description = Infraction ID
     .add-severity = severity
     .add-severity-description = Infraction severity
     .add-severity-Low = Low severity
     .add-severity-Mid = Mid severity
     .add-severity-High = High severity
     .add-punishment = punishment
     .add-punishment-description = Infraction punishment
     .add-punishment-Strike = Strike user
     .add-punishment-Timeout = Timeout user
     .add-punishment-Ban = Ban user
     .add-punishment-Kick = Kick user
     .add-duration = duration
     .add-duration-description = Duration of the timeout
     .list = list
     .list-description = Infraction table
     .remove = remove
     .remove-description = Remove infraction from the table
     .remove-id = id
     .remove-id-description = ID of the infraction to be removed
     .user = user
     .user-description = List infractions of a specific user
     .user-member = member
     .user-member-description = Member of the guild

kick = kick
     .description = Kick users
     .users = users
     .users-description = Users to be kicked
     .reason = reason
     .reason-description = Reason to kick users

timeout = timeout
     .description = Time out users for a specific amount of time
     .users = users
     .users-description = Users to be timed out
     .time = time
     .time-description = Time in numbers
     .unit = unit
     .unit-description = Unit of time
     .unit-Seconds = Seconds (time * 1)
     .unit-Minutes = Minutes (time * 60)
     .unit-Hours = Hours (time * 60 * 60)
     .unit-Days = Days (time * 60 * 60 * 24)

untimeout = untimeout
     .description = Remove time out from users
     .users = users
     .users-description = Users to have time out removed

ban = ban
     .description = Ban users from the server
     .users = users
     .users-description = Users to be banned
     .reason = reason
     .reason-description = Reason to ban users

unban = unban
     .description = Unban users from the server
     .users = users
     .users-description = Users to be unbanned

strike = strike
     .description = Strike users
     .users = users
     .users-description = Users to be striked
     .reason = reason
     .reason-description = Reason to strike users

punish = punish
     .description = Punish users
     .id = id
     .id-description = Infraction ID
     .users = users
     .users-description = Users to be punished
     .message = message
     .message-description = Reason to punish users

tag = tag
    .description = Tag command
    .add = add
    .add-description = Create a new tag
    .add-name = name
    .add-name-description = Name of the new tag
    .add-content = content
    .add-content-description = Content of the new tag
    .edit = edit
    .edit-description = Edit an existing tag you own
    .edit-name = name
    .edit-name-description = Name of the tag you want to edit
    .edit-content = content
    .edit-content-description = New content of the tag
    .see = see
    .see-description = Show the contents of a specific tag
    .see-name = name
    .see-name-description = Name of the tag you want to see the contents
    .list = list
    .list-description = List server tags
    .user = user
    .user-description = List user tags
    .user-user = user
    .user-user-description = User to fetch tags from
    .remove = remove
    .remove-description = Delete a tag you own
    .remove-name = name
    .remove-name-description = Name of the tag you want to delete

emoji = emoji
    .description = Emoji command
    .see = see
    .see-description = Show the emoji image
    .see-emoji = emoji
    .see-emoji-description = Emoji you want to see the image
    .add = add
    .add-description = Add a new emoji to the server
    .add-name = name
    .add-name-description = Name of the new emoji
    .add-attachment = attachment
    .add-attachment-description = Emoji image
    .list = list
    .list-description = List server emojis
    .remove = remove
    .remove-description = Remove emoji from the server
    .remove-emoji = emoji
    .remove-emoji-description = Emoji to be removed

# Responses
Pong = Pong! :ping_pong: