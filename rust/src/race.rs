use crate::data_loader::RACE_STATUS_GRADES;
use crate::status::{Grade, StatusKind};

use clap::ValueEnum;
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter, VariantArray, ValueEnum, Enum, Serialize, Deserialize)]
pub enum Race {
    Hum,
    Elv,
    Tar,
    Mit,
    Gal,
}

impl Race {
    pub fn status_grade(&self, kind: StatusKind) -> Grade {
        RACE_STATUS_GRADES[*self][kind]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_grade_all_cases() {
        // 全種族 × 全 stat の値が JSON 由来データで取得できることを確認
        for race in Race::VARIANTS {
            for status in StatusKind::VARIANTS {
                let _ = race.status_grade(*status);
            }
        }
    }

    #[test]
    fn test_status_grade_known_values() {
        // 代表値の回帰テスト: 旧 STATUS_GRADES から不変であることを担保
        assert_eq!(Race::Hum.status_grade(StatusKind::Hp), Grade::D);
        assert_eq!(Race::Elv.status_grade(StatusKind::Str), Grade::B);
        assert_eq!(Race::Tar.status_grade(StatusKind::Int), Grade::A);
        assert_eq!(Race::Mit.status_grade(StatusKind::Dex), Grade::A);
        assert_eq!(Race::Gal.status_grade(StatusKind::Hp), Grade::A);
    }
}
