use bancho_protocol::structures::Privileges as BanchoPrivileges;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct Privileges: i32 {
        const None = 0;
        const PubliclyVisible = 1 << 0;
        const CanLogin = 1 << 1;
        const Donator = 1 << 2;
        const AdminAccessPanel = 1 << 3;
        const AdminManageUsers = 1 << 4;
        const AdminManageBans = 1 << 5;
        const AdminSilenceUsers = 1 << 6;
        const AdminWipeUsers = 1 << 7;
        const AdminManageBeatmaps = 1 << 8;
        const AdminManageServers = 1 << 9;
        const AdminManageSettings = 1 << 10;
        const AdminManageBetakeys = 1 << 11;
        const AdminManageReports = 1 << 12;
        const AdminManageDocs = 1 << 13;
        const AdminManageBadges = 1 << 14;
        const AdminViewAuditLogs = 1 << 15;
        const AdminManagePrivileges = 1 << 16;
        const AdminSendAlerts = 1 << 17;
        const AdminChatMod = 1 << 18;
        const AdminKickUsers = 1 << 19;
        const PendingVerification = 1 << 20;
        const AdminTournamentStaff = 1 << 21;
        const AdminCaker = 1 << 22;
        const AkatsukiPlus = 1 << 23;
        const AdminFreezeUsers = 1 << 24;
        const AdminManageNominators = 1 << 25;
    }
}

impl Privileges {
    pub fn is_publicly_visible(&self) -> bool {
        self.contains(Privileges::PubliclyVisible)
    }

    pub fn is_donor(&self) -> bool {
        self.intersects(Privileges::Donator | Privileges::AkatsukiPlus)
    }

    pub fn is_staff(&self) -> bool {
        self.contains(Privileges::AdminChatMod)
    }

    pub fn is_admin(&self) -> bool {
        self.contains(Privileges::AdminCaker)
    }

    pub fn is_developer(&self) -> bool {
        self.contains(Privileges::AdminManagePrivileges)
    }

    pub fn is_tournament_staff(&self) -> bool {
        self.contains(Privileges::AdminTournamentStaff)
    }

    pub fn to_bancho(&self) -> BanchoPrivileges {
        let mut privileges = BanchoPrivileges::None;
        if self.is_publicly_visible() {
            privileges |= BanchoPrivileges::Player;
        }

        if self.is_donor() {
            privileges |= BanchoPrivileges::Supporter;
        }
        if self.is_staff() {
            privileges |= BanchoPrivileges::Moderator;
        }

        if self.is_admin() {
            privileges |= BanchoPrivileges::LeGuy;
        } else if self.is_developer() {
            privileges |= BanchoPrivileges::Developer;
        }

        if self.is_tournament_staff() {
            privileges |= BanchoPrivileges::TournamentStaff;
        }

        privileges
    }
}
