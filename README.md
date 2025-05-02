### My first ever actual rust program (and yes, it's a discord bot for my game community.)
People are saying that it's hard to write bad code in Rust, this project proves 'em wrong. 😼

## .ENV ->
```
LOOPCHAN_DISCORD_TOKEN
PTL_GUILD_ID
OWNER_ID
STAFF_ROLE_ID
QA_ROLE_ID
QA_FORM_LINK
MEMBER_ROLE_ID
ERROR_CHANNEL_ID
QA_FORMS_CHANNEL_ID
WELCOME_CHANNEL_ID
DATABASE_PATH
ALL_COMMANDS_COOLDOWN
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