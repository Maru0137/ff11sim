use crate::status::{Grade, StatusKind};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray, ValueEnum, Serialize, Deserialize)]
pub enum Race {
    Hum,
    Elv,
    Tar,
    Mit,
    Gal,
}

const STATUS_GRADES: [[Grade; StatusKind::COUNT]; Race::COUNT] = [
    [
        Grade::D,
        Grade::D,
        Grade::D,
        Grade::D,
        Grade::D,
        Grade::D,
        Grade::D,
        Grade::D,
        Grade::D,
    ],
    [
        Grade::C,
        Grade::E,
        Grade::B,
        Grade::E,
        Grade::C,
        Grade::F,
        Grade::F,
        Grade::B,
        Grade::D,
    ],
    [
        Grade::G,
        Grade::A,
        Grade::F,
        Grade::D,
        Grade::E,
        Grade::C,
        Grade::A,
        Grade::E,
        Grade::D,
    ],
    [
        Grade::D,
        Grade::D,
        Grade::E,
        Grade::A,
        Grade::E,
        Grade::B,
        Grade::D,
        Grade::E,
        Grade::F,
    ],
    [
        Grade::A,
        Grade::G,
        Grade::C,
        Grade::D,
        Grade::A,
        Grade::E,
        Grade::E,
        Grade::D,
        Grade::F,
    ],
];

impl Race {
    pub fn status_grade(&self, kind: StatusKind) -> Grade {
        return STATUS_GRADES[*self as usize][kind as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_grade_all_cases() {
        for race in Race::VARIANTS {
            for status in StatusKind::VARIANTS {
                let expected = STATUS_GRADES[*race as usize][*status as usize];
                let actual = race.status_grade(*status);

                assert_eq!(
                    actual, expected,
                    "Failed for Race: {:?}, StatusKind: {:?}",
                    race, status
                );
            }
        }
    }
}
