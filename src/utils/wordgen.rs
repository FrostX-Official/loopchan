use rand::Rng;

const WORDS_STR: &str = "data
lose
win
ragequit
perpetuity
frost
prokitek
cheaterban
parkour
loop
parkourtheloop
ptl
community
ranked
movement
competitive
timetrial
otimads
kapitanwai
datalose
kremle
ash
joe
quilical
alver
kiwii
nightplay
dorkk
wallrun
wallclimb
speedvault
dash
grappler
wingsuit
quickturn
genesis
elo
bloxycola
slide
route
astrid
gap
silly
adrenaline
combo
flow
leaderboard
gearless
spawn
zipline
vertex
velocity
maro
geum
jejo
saeng
daehu
donch
hyeon
poslar
elysium
gear
party
vector
prop
vent
landing
bronze
silver
gold
platinum
diamond
master
elite
points
skins
hwlq
vecetyp
longjump
coil
springboard
cutdash
stimreset
stimhop
paraglider
trinket
deka
fangs
skydive
ledgegrab
ledgeboost
ziplaunch
downshift";

pub async fn getgenwords() -> Vec<&'static str> {
    return WORDS_STR.split("\n").collect();
}

pub async fn getrandomgenword() -> String {
    let words: Vec<&str> = getgenwords().await;
    let num: usize = rand::rng().random_range(0..words.len());
    words[num].to_string()
}