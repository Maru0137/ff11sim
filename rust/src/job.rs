use crate::status::{Grade, StatusKind};
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray, Enum, Serialize, Deserialize)]
pub enum Job {
    War,
    Mnk,
    Whm,
    Blm,
    Rdm,
    Thf,
    Pld,
    Drk,
    Bst,
    Brd,
    Rng,
    Sam,
    Nin,
    Drg,
    Smn,
    Blu,
    Cor,
    Pup,
    Dnc,
    Sch,
    Geo,
    Run,
}

// Job status grades: [HP, MP, STR, DEX, VIT, AGI, INT, MND, CHR]
// None means the job doesn't contribute to that stat (e.g., no MP for melee jobs)
// Data from: https://wiki.ffo.jp/html/316.html
const JOB_STATUS_GRADES: [[Option<Grade>; StatusKind::COUNT]; Job::COUNT] = [
    // War: HP=B, MP=-, STR=A, DEX=C, VIT=D, AGI=C, INT=F, MND=F, CHR=E
    [Some(Grade::B), None, Some(Grade::A), Some(Grade::C), Some(Grade::D), Some(Grade::C), Some(Grade::F), Some(Grade::F), Some(Grade::E)],
    // Mnk: HP=A, MP=-, STR=C, DEX=B, VIT=A, AGI=F, INT=G, MND=D, CHR=E
    [Some(Grade::A), None, Some(Grade::C), Some(Grade::B), Some(Grade::A), Some(Grade::F), Some(Grade::G), Some(Grade::D), Some(Grade::E)],
    // Whm: HP=E, MP=C, STR=D, DEX=F, VIT=D, AGI=E, INT=E, MND=A, CHR=C
    [Some(Grade::E), Some(Grade::C), Some(Grade::D), Some(Grade::F), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::A), Some(Grade::C)],
    // Blm: HP=F, MP=B, STR=F, DEX=C, VIT=F, AGI=C, INT=A, MND=E, CHR=D
    [Some(Grade::F), Some(Grade::B), Some(Grade::F), Some(Grade::C), Some(Grade::F), Some(Grade::C), Some(Grade::A), Some(Grade::E), Some(Grade::D)],
    // Rdm: HP=D, MP=D, STR=D, DEX=D, VIT=E, AGI=E, INT=C, MND=C, CHR=D
    [Some(Grade::D), Some(Grade::D), Some(Grade::D), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::C), Some(Grade::C), Some(Grade::D)],
    // Thf: HP=D, MP=-, STR=D, DEX=A, VIT=D, AGI=B, INT=C, MND=G, CHR=G
    [Some(Grade::D), None, Some(Grade::D), Some(Grade::A), Some(Grade::D), Some(Grade::B), Some(Grade::C), Some(Grade::G), Some(Grade::G)],
    // Pld: HP=C, MP=F, STR=B, DEX=E, VIT=A, AGI=G, INT=G, MND=C, CHR=C
    [Some(Grade::C), Some(Grade::F), Some(Grade::B), Some(Grade::E), Some(Grade::A), Some(Grade::G), Some(Grade::G), Some(Grade::C), Some(Grade::C)],
    // Drk: HP=C, MP=F, STR=A, DEX=C, VIT=C, AGI=D, INT=C, MND=G, CHR=G
    [Some(Grade::C), Some(Grade::F), Some(Grade::A), Some(Grade::C), Some(Grade::C), Some(Grade::D), Some(Grade::C), Some(Grade::G), Some(Grade::G)],
    // Bst: HP=C, MP=-, STR=D, DEX=C, VIT=D, AGI=F, INT=E, MND=E, CHR=A
    [Some(Grade::C), None, Some(Grade::D), Some(Grade::C), Some(Grade::D), Some(Grade::F), Some(Grade::E), Some(Grade::E), Some(Grade::A)],
    // Brd: HP=D, MP=-, STR=D, DEX=D, VIT=D, AGI=F, INT=D, MND=D, CHR=B
    [Some(Grade::D), None, Some(Grade::D), Some(Grade::D), Some(Grade::D), Some(Grade::F), Some(Grade::D), Some(Grade::D), Some(Grade::B)],
    // Rng: HP=E, MP=-, STR=E, DEX=D, VIT=D, AGI=A, INT=E, MND=D, CHR=E
    [Some(Grade::E), None, Some(Grade::E), Some(Grade::D), Some(Grade::D), Some(Grade::A), Some(Grade::E), Some(Grade::D), Some(Grade::E)],
    // Sam: HP=B, MP=-, STR=C, DEX=C, VIT=C, AGI=D, INT=E, MND=E, CHR=D
    [Some(Grade::B), None, Some(Grade::C), Some(Grade::C), Some(Grade::C), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::D)],
    // Nin: HP=D, MP=-, STR=C, DEX=B, VIT=C, AGI=B, INT=D, MND=G, CHR=F
    [Some(Grade::D), None, Some(Grade::C), Some(Grade::B), Some(Grade::C), Some(Grade::B), Some(Grade::D), Some(Grade::G), Some(Grade::F)],
    // Drg: HP=B, MP=-, STR=B, DEX=D, VIT=C, AGI=D, INT=F, MND=E, CHR=C
    [Some(Grade::B), None, Some(Grade::B), Some(Grade::D), Some(Grade::C), Some(Grade::D), Some(Grade::F), Some(Grade::E), Some(Grade::C)],
    // Smn: HP=G, MP=A, STR=F, DEX=E, VIT=F, AGI=D, INT=B, MND=B, CHR=B
    [Some(Grade::G), Some(Grade::A), Some(Grade::F), Some(Grade::E), Some(Grade::F), Some(Grade::D), Some(Grade::B), Some(Grade::B), Some(Grade::B)],
    // Blu: HP=D, MP=D, STR=E, DEX=E, VIT=E, AGI=E, INT=E, MND=E, CHR=E
    [Some(Grade::D), Some(Grade::D), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E), Some(Grade::E)],
    // Cor: HP=D, MP=-, STR=E, DEX=C, VIT=E, AGI=B, INT=C, MND=E, CHR=E
    [Some(Grade::D), None, Some(Grade::E), Some(Grade::C), Some(Grade::E), Some(Grade::B), Some(Grade::C), Some(Grade::E), Some(Grade::E)],
    // Pup: HP=D, MP=-, STR=E, DEX=B, VIT=D, AGI=C, INT=E, MND=F, CHR=C
    [Some(Grade::D), None, Some(Grade::E), Some(Grade::B), Some(Grade::D), Some(Grade::C), Some(Grade::E), Some(Grade::F), Some(Grade::C)],
    // Dnc: HP=D, MP=-, STR=D, DEX=C, VIT=E, AGI=B, INT=F, MND=F, CHR=B
    [Some(Grade::D), None, Some(Grade::D), Some(Grade::C), Some(Grade::E), Some(Grade::B), Some(Grade::F), Some(Grade::F), Some(Grade::B)],
    // Sch: HP=E, MP=D, STR=F, DEX=D, VIT=E, AGI=D, INT=B, MND=D, CHR=C
    [Some(Grade::E), Some(Grade::D), Some(Grade::F), Some(Grade::D), Some(Grade::E), Some(Grade::D), Some(Grade::B), Some(Grade::D), Some(Grade::C)],
    // Geo: HP=D, MP=C, STR=F, DEX=D, VIT=D, AGI=E, INT=B, MND=B, CHR=E
    [Some(Grade::D), Some(Grade::C), Some(Grade::F), Some(Grade::D), Some(Grade::D), Some(Grade::E), Some(Grade::B), Some(Grade::B), Some(Grade::E)],
    // Run: HP=B, MP=F, STR=C, DEX=D, VIT=E, AGI=B, INT=D, MND=D, CHR=F
    [Some(Grade::B), Some(Grade::F), Some(Grade::C), Some(Grade::D), Some(Grade::E), Some(Grade::B), Some(Grade::D), Some(Grade::D), Some(Grade::F)],
];

impl Job {
    pub fn status_grade(&self, kind: StatusKind) -> Option<Grade> {
        JOB_STATUS_GRADES[*self as usize][kind as usize]
    }
}
