### My first ever actual rust program (and yes, it's a discord bot for my game community.)
People are saying that it's hard to write bad code in Rust, this project proves 'em wrong. 😼

## .ENV ->
```
LOOPCHAN_DISCORD_TOKEN=String

PTL_GUILD_ID=Integer

QA_ROLE_ID=Integer
STAFF_ROLE_ID=Integer
MEMBER_ROLE_ID=Integer

OWNER_ID=Integer
ERROR_CHANNEL_ID=Integer
QA_FORMS_CHANNEL_ID=Integer
UNVERIFIED_CHAT_CHANNEL_ID=Integer

QA_FORM_LINK=String

ALL_COMMANDS_COOLDOWN=Integer

WELCOME_CHANNEL_ID=Integer
WELCOME_MESSAGE_EMOJI_ID=Integer
WELCOME_MESSAGE_EMOJI_NAME=String
WELCOME_MESSAGE_EMOJI_ANIMATED=Integer (0 or 1)

DATABASE_PATH=String (Optional)
```

## TODOs ->
- Add cooldown to every command in main.rs -> handle_slash_command
- Economic (loopcoins/tokens or smth and user level & exp)
- Giveaways
- Reaction Roles

## Unsure ->
- Moderation
- Logs
<br>
Moving all moderation and logs handling to loopchan can make it a bit heavy for current machine im using to host loopchan on, so not in main priority (probably never will be added)