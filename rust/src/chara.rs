use std::option::Option;

use crate::job::Job;
use crate::race::Race;
use crate::status::{calc_master_lv_bonus, calc_status, StatusKind};

#[derive(Debug, Clone)]
pub struct Chara {
    pub race: Race,
    pub main_job: Job,
    pub main_lv: i32,
    pub support_job: Option<Job>,
    pub support_lv: Option<i32>,
    pub master_lv: i32,
}

impl Chara {
    pub fn builder() -> CharaBuilder {
        CharaBuilder::default()
    }

    pub fn status(&self, kind: StatusKind) -> i32 {
        // For MP: if main job has no MP, return 0 (no race/support/mlv contribution)
        if kind == StatusKind::Mp && self.main_job.status_grade(StatusKind::Mp).is_none() {
            return 0;
        }

        // Race status
        let grade_race = self.race.status_grade(kind);
        let status_race = calc_status(kind, grade_race, self.main_lv);

        // Main job status
        let status_main_job = match self.main_job.status_grade(kind) {
            Some(grade) => calc_status(kind, grade, self.main_lv),
            None => 0.0,
        };

        // Support job status (calculated at support_lv, then halved)
        let status_support_job = match (&self.support_job, &self.support_lv) {
            (Some(job), Some(lv)) => match job.status_grade(kind) {
                Some(grade) => calc_status(kind, grade, *lv) / 2.0,
                None => 0.0,
            },
            _ => 0.0,
        };

        // Master level bonus
        let mlv_bonus = calc_master_lv_bonus(kind, self.master_lv);

        (status_race + status_main_job + status_support_job).floor() as i32 + mlv_bonus
    }
}

#[derive(Default)]
pub struct CharaBuilder {
    race: Option<Race>,
    main_job: Option<Job>,
    main_lv: Option<i32>,
    support_job: Option<Job>,
    support_lv: Option<i32>,
    master_lv: Option<i32>,
}

impl CharaBuilder {
    pub fn race(mut self, race: Race) -> Self {
        self.race = Some(race);
        self
    }

    pub fn main_job(mut self, job: Job, lv: i32) -> Self {
        assert!(lv > 0 && lv <= 99, "main_lv must be between 1 and 99");
        self.main_job = Some(job);
        self.main_lv = Some(lv);
        self
    }

    pub fn support_job(mut self, job: Job, lv: i32) -> Self {
        assert!(lv > 0 && lv <= 99, "support_lv must be between 1 and 99");
        self.support_job = Some(job);
        self.support_lv = Some(lv);
        self
    }

    pub fn master_lv(mut self, master_lv: i32) -> Self {
        assert!(
            master_lv >= 0 && master_lv <= 50,
            "master_lv must be between 0 and 50"
        );
        self.master_lv = Some(master_lv);
        self
    }

    pub fn build(self) -> Result<Chara, &'static str> {
        Ok(Chara {
            race: self.race.ok_or("race is required")?,
            main_job: self.main_job.ok_or("main_job is required")?,
            main_lv: self.main_lv.ok_or("main_lv is required")?,
            support_job: self.support_job,
            support_lv: self.support_lv,
            master_lv: self.master_lv.ok_or("master_lv is required")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chara_builder_success() {
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Drg, 59)
            .master_lv(50)
            .build()
            .expect("Failed to build Chara");

        assert_eq!(chara.race, Race::Hum);
        assert_eq!(chara.main_job, Job::War);
        assert_eq!(chara.main_lv, 99);
        assert_eq!(chara.support_job, Some(Job::Drg));
        assert_eq!(chara.support_lv, Some(59));
        assert_eq!(chara.master_lv, 50);
    }

    #[test]
    fn test_chara_builder_missing_required_fields() {
        let result = Chara::builder().race(Race::Hum).build();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "main_job is required");
    }

    #[test]
    fn test_chara_builder_default_support_job_and_lv() {
        let chara = Chara::builder()
            .race(Race::Tar)
            .main_job(Job::Blm, 90)
            .master_lv(50)
            .build()
            .expect("Failed to build Chara");

        assert_eq!(chara.support_job, None);
        assert_eq!(chara.support_lv, None);
    }

    #[test]
    fn test_chara_status_war_drg() {
        // Hum/War99/Drg/MLV50
        // Support calc lv = 99/2 + 50/5 = 49 + 10 = 59
        // HP = race(D:485) + job(B:675) + support(B@59:510/2=255) + mlv(350) = 1765
        // STR = race(D:37.5) + job(A:45) + support(B@59:30/2=15) + mlv(50) = 147
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Drg, 59)
            .master_lv(50)
            .build()
            .expect("Failed to build Chara");

        assert_eq!(chara.status(StatusKind::Hp), 1765);
        assert_eq!(chara.status(StatusKind::Str), 147);
        // War has no MP grade, so MP should be 0 (no MLV bonus either)
        assert_eq!(chara.status(StatusKind::Mp), 0);
    }

    #[test]
    fn test_chara_status_blm_with_mp() {
        // Tar/Blm99/Rdm@59/MLV50
        // Tar has MP grade A, Blm has MP grade B, Rdm has MP grade D
        let chara = Chara::builder()
            .race(Race::Tar)
            .main_job(Job::Blm, 99)
            .support_job(Job::Rdm, 59)
            .master_lv(50)
            .build()
            .expect("Failed to build Chara");

        // MP should be non-zero since Blm has MP
        assert!(chara.status(StatusKind::Mp) > 0);
        // MLV MP bonus should be applied (2 * 50 = 100)
        // Tar(A:736) + Blm(B:675) + Rdm(D@59:362/2=181) + mlv(100) = 1692
        assert_eq!(chara.status(StatusKind::Mp), 1692);
    }

    #[test]
    fn test_chara_status_no_support_job() {
        // Test without support job
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .master_lv(0)
            .build()
            .expect("Failed to build Chara");

        // HP = race(D:485) + job(B:675) = 1160
        assert_eq!(chara.status(StatusKind::Hp), 1160);
        // STR = race(D:37.5) + job(A:45) = 82
        assert_eq!(chara.status(StatusKind::Str), 82);
    }
}
