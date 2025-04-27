use rand::Rng;

const WORDS_STR: &str = "data
lose
perpetuity
hudzell
lunari
frost
prokitek
parkour
loop
parkour the loop
community
reborn
ranked
movement
competitive
time trial
otimads
kapitan wai
datalose
kremble
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
bloxy cola
slide
route
gap
silly
infinite map
adrenaline
combo
flow
leaderboard
gearless
spawn
zipline
vertex
velocity
gear
party
prop
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
long jump
coil
springboard
cut dash
stim reset
stim hop
paraglider
skydive
dropbug
downshift
downslam";

pub async fn getgenwords() -> Vec<&'static str> {
    return WORDS_STR.split("\n").collect();
}

pub async fn getrandomgenword() -> String {
    let words: Vec<&str> = getgenwords().await;
    let num: usize = rand::rng().random_range(0..words.len());
    words[num].to_string()
}