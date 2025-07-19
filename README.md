### My first ever actual rust program (and yes, it's a discord bot for my game community.)
People are saying that it's hard to write bad code in Rust, this project proves 'em wrong. 😼

## .ENV ->
```py
LOOPCHAN_DISCORD_TOKEN=String
QA_FORM_LINK=String
LAST_FM_API_KEY=String
LASM_FM_API_SECRET=String
```

## Config ->
Look into [Config.toml](/Config.toml) for description of config variables.

## TODOs ->
* *l18n?*
* Counting channel moderation
* Add permission check back to bot, incase discord goes stupid
* Economics
  * [x] Balance
  * [x] Exp for chatting
  * [x] Level System
  * [x] Leaderboards
  * [x] Work for coins commands *(with cooldown like 30 minutes)*
  * [x] Custom Role Shop
  * [ ] Paying coinys to others
  * [ ] More admin commands (like customizing balance)
  * Fishing (😍)
    * [x] Actual fishing
    * [x] Fish Inventory
    * [x] Fish DB
    * [ ] Fish Trading
* Last.fm
  * [x] Authorization
  * [ ] Get info about tracks, albums & artists commands
  * [ ] Server leaderboard for like specific artist
* Giveaways
* esmBot Replacement
  * [ ] Convert image to GIF command
  * [ ] Add caption to image command
  * [ ] Add speechbubble to image command
* New Design
  * [ ] Profile Picture
  * [ ] Banner

## Unsure ->
* Moderation
* Logs
<br><br>Moving all moderation and logs handling to loopchan can make it a bit heavy for current machine im using to host loopchan on, so not in main priority (probably never will be added)