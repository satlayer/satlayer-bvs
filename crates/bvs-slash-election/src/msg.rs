use bvs_slash_base::{
    self,
    msg::{ExecuteSlash, SubmitSlash},
};

use crate::state::Offense;

pub struct SlashDetails {
    pub offender: String,
    pub offense: Offense,
    pub start_height: u64,
}

pub type SubmitSlashMsg = SubmitSlash<Offense, Option<String>>;

pub type ExecuteSlashMsg = ExecuteSlash<SlashDetails>;

pub struct VoteSlashMsg {
    slash: SlashDetails,
    approve: bool,
}
pub struct SetPunishmentMsg {
    slash: SlashDetails,
    approve: bool,
}

pub enum ExecuteMsg {
    SubmitSlash(SubmitSlashMsg),

    VoteSlash(VoteSlashMsg),

    ExecuteSlash(ExecuteSlashMsg),

    SetPunishment(SetPunishmentMsg),

    SetThreshold(u64),
}

pub enum QueryMsg {}

pub struct InstantiateMsg {
    pub pauser: String,
    pub router: String,
    pub registry: String,
    pub owner: String,
    pub threshold: u64,
}
