use std::option::Option;

use crate::job::Job;
use crate::race::Race;

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
            "main_lv must be between 1 and 99"
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
}
