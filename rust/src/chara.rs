use std::option::Option;

use crate::job::{blu_trait_effect_up_bonus_ranks, Job, JobTrait};
use crate::job_points::JobPointCategories;
use crate::race::Race;
use crate::skills::CharacterSkills;
use crate::status::{calc_master_lv_bonus, calc_status, BonusStats, MeritPoints, StatusKind};

#[derive(Debug, Clone)]
pub struct Chara {
    pub race: Race,
    pub main_job: Job,
    pub main_lv: i32,
    pub support_job: Option<Job>,
    pub support_lv: Option<i32>,
    pub master_lv: i32,
    pub merit_points: MeritPoints,
    pub bonus_stats: BonusStats,
    /// メインジョブのジョブポイントカテゴリ
    pub job_points: JobPointCategories,
    /// キャラクター共通のスキル値
    pub skills: CharacterSkills,
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

        // Merit point bonus
        let merit_bonus = self.merit_points.status_bonus(kind);

        // Job trait bonus for HP/MP
        let trait_hp_mp = match kind {
            StatusKind::Hp => {
                self.job_trait_total(JobTrait::MaxHpBoost)
                    + self.job_trait_total(JobTrait::MaxHpBoost2)
            }
            StatusKind::Mp => self.job_trait_total(JobTrait::MaxMpBoost),
            _ => 0,
        };

        (status_race + status_main_job + status_support_job).floor() as i32
            + mlv_bonus
            + merit_bonus
            + self.bonus_stats.get(kind)
            + trait_hp_mp
    }

    /// Calculate total job trait bonus from main + support job.
    /// メインジョブが BLU の場合、ギフト「ジョブ特性効果アップ」(100JP=+1, 1200JP=+2 ランク)
    /// を base rank に加算する (除外特性: Gilfinder/DoubleAttack/AutoRefresh/TripleAttack)。
    ///
    /// 集約規則: 「効果の強い方」を採用。通常特性 (正値) は max、
    /// MartialArts のような負値特性 (隔短縮) は min を取りたいため、
    /// 単純に絶対値が大きい方を選ぶ (符号は同一前提)。
    pub fn job_trait_total(&self, trait_kind: JobTrait) -> i32 {
        let main = self.main_job_trait_bonus(trait_kind);
        let support = match (&self.support_job, &self.support_lv) {
            (Some(job), Some(lv)) => job.trait_bonus(trait_kind, *lv),
            _ => 0,
        };
        if main.abs() >= support.abs() {
            main
        } else {
            support
        }
    }

    /// メインジョブ単独のジョブ特性ボーナス (BLU ギフトを考慮)。
    fn main_job_trait_bonus(&self, trait_kind: JobTrait) -> i32 {
        let base_rank = self.main_job.trait_rank_at_lv(trait_kind, self.main_lv);
        if base_rank == 0 {
            // 未習得特性にはギフトのランクアップは適用されない
            return 0;
        }
        let bonus_rank = if self.main_job == Job::Blu && !trait_kind.is_blu_effect_up_excluded() {
            blu_trait_effect_up_bonus_ranks(self.job_points.total_jp_spent())
        } else {
            0
        };
        trait_kind.value_at_rank(base_rank + bonus_rank)
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
    merit_points: MeritPoints,
    bonus_stats: BonusStats,
    job_points: JobPointCategories,
    skills: CharacterSkills,
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

    pub fn merit_points(mut self, merit_points: MeritPoints) -> Self {
        self.merit_points = merit_points;
        self
    }

    pub fn bonus_stats(mut self, bonus_stats: BonusStats) -> Self {
        self.bonus_stats = bonus_stats;
        self
    }

    pub fn job_points(mut self, job_points: JobPointCategories) -> Self {
        self.job_points = job_points;
        self
    }

    pub fn skills(mut self, skills: CharacterSkills) -> Self {
        self.skills = skills;
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
            merit_points: self.merit_points,
            bonus_stats: self.bonus_stats,
            job_points: self.job_points,
            skills: self.skills,
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
        // HP = race(D:485) + job(B:675) + support(B@59:510/2=255) + mlv(350) + trait(180) = 1945
        // STR = race(D:37.5) + job(A:45) + support(B@59:30/2=15) + mlv(50) = 147
        let chara = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::War, 99)
            .support_job(Job::Drg, 59)
            .master_lv(50)
            .build()
            .expect("Failed to build Chara");

        assert_eq!(chara.status(StatusKind::Hp), 1945);
        assert_eq!(chara.status(StatusKind::Str), 147);
        // War has no MP grade, so MP should be 0 (no MLV bonus either)
        assert_eq!(chara.status(StatusKind::Mp), 0);
    }

    #[test]
    fn test_chara_status_cor_sam() {
        // Gal/Cor99/Sam/MLV50
        // Support calc lv = 99/2 + 50/5 = 49 + 10 = 59
        let chara = Chara::builder()
            .race(Race::Gal)
            .main_job(Job::Cor, 99)
            .support_job(Job::Sam, 59)
            .master_lv(50)
            .build()
            .expect("Failed to build Chara");

        assert_eq!(chara.status(StatusKind::Str), 138);
        assert_eq!(chara.status(StatusKind::Dex), 141);
        assert_eq!(chara.status(StatusKind::Vit), 143);
        assert_eq!(chara.status(StatusKind::Agi), 138);
        assert_eq!(chara.status(StatusKind::Int), 135);
        assert_eq!(chara.status(StatusKind::Mnd), 132);
        assert_eq!(chara.status(StatusKind::Chr), 127);
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

        // HP = race(D:485) + job(B:675) + trait(180) = 1340
        assert_eq!(chara.status(StatusKind::Hp), 1340);
        // STR = race(D:37.5) + job(A:45) = 82
        assert_eq!(chara.status(StatusKind::Str), 82);
    }

    // -----------------------------------------------------------------------
    // BLU ギフト「ジョブ特性効果アップ」(https://wiki.ffo.jp/html/34014.html)
    // 100 JP = +1 rank, 1200 JP = +2 rank
    // 例外: Gilfinder / DoubleAttack / AutoRefresh / TripleAttack
    // -----------------------------------------------------------------------

    /// total_jp >= target_jp を満たす最小構成の JobPointCategories を返す。
    /// JP コストは rank r ごとに r*(r+1)/2、各カテゴリ rank 0..=20。
    fn build_jp_categories_with_at_least(target_jp: i32) -> JobPointCategories {
        let mut jpc = JobPointCategories::default();
        let mut total = 0;
        for rank in jpc.ranks.iter_mut() {
            if total >= target_jp {
                break;
            }
            // このカテゴリで rank r まで上げると cost r*(r+1)/2
            // target を超える最小 r を選ぶ
            let need = target_jp - total;
            let mut r = 0;
            while r < 20 && r * (r + 1) / 2 < need {
                r += 1;
            }
            *rank = r;
            total += r * (r + 1) / 2;
        }
        jpc
    }

    fn build_blu99_with_jp(target_jp: i32) -> Chara {
        let jpc = build_jp_categories_with_at_least(target_jp);
        Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Blu, 99)
            .master_lv(0)
            .job_points(jpc)
            .build()
            .expect("Failed to build BLU")
    }

    #[test]
    fn test_blu_magic_accuracy_bonus_no_gift() {
        // BLU99, 0 JP → rank 1 = +10
        let chara = build_blu99_with_jp(0);
        assert_eq!(chara.job_trait_total(JobTrait::MagicAccuracyBonus), 10);
        assert_eq!(chara.job_trait_total(JobTrait::MagicEvasionBonus), 10);
    }

    #[test]
    fn test_blu_magic_accuracy_bonus_gift_100jp() {
        // BLU99, 100 JP → rank 1 + 1 = rank 2 = +22 (累積)
        let chara = build_blu99_with_jp(100);
        assert_eq!(chara.job_trait_total(JobTrait::MagicAccuracyBonus), 22);
        assert_eq!(chara.job_trait_total(JobTrait::MagicEvasionBonus), 22);
    }

    #[test]
    fn test_blu_magic_accuracy_bonus_gift_1200jp() {
        // BLU99, 1200 JP → rank 1 + 2 = rank 3
        // wiki に rank 3 値の記載がないため累積配列 [10,22] で clamp → 22
        let chara = build_blu99_with_jp(1200);
        assert_eq!(chara.job_trait_total(JobTrait::MagicAccuracyBonus), 22);
        assert_eq!(chara.job_trait_total(JobTrait::MagicEvasionBonus), 22);
    }

    #[test]
    fn test_blu_double_attack_excluded_from_gift() {
        // BLU99 は DoubleAttack rank 1 (Lv80) を持つが、ギフト除外特性のためランクアップしない。
        // DOUBLE_ATTACK = [10, 12, 14, 16, 18] → rank 1 = 10
        let chara0 = build_blu99_with_jp(0);
        let chara1200 = build_blu99_with_jp(1200);
        assert_eq!(chara0.job_trait_total(JobTrait::DoubleAttack), 10);
        // ギフト除外なので 1200 JP でも値は変わらず 10 のまま
        assert_eq!(chara1200.job_trait_total(JobTrait::DoubleAttack), 10);
    }

    #[test]
    fn test_non_blu_main_no_gift_effect() {
        // SAM99 (非 BLU) には「ジョブ特性効果アップ」ギフトは適用されない。
        // SAM の StoreTp = rank 5 = 30 (Lv99, 配列末尾)、JP の有無で値が変わらないこと。
        let sam_no_jp = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Sam, 99)
            .master_lv(0)
            .build()
            .unwrap();
        let sam_full_jp = Chara::builder()
            .race(Race::Hum)
            .main_job(Job::Sam, 99)
            .master_lv(0)
            .job_points(JobPointCategories::all_maxed())
            .build()
            .unwrap();
        assert_eq!(sam_no_jp.job_trait_total(JobTrait::StoreTp), 30);
        // ※ Store TP のジョブ特性自体は変わらず 30。JP カテゴリ「ストアTP」は
        //   wasm 側で別途加算されるが、job_trait_total としては 30 のまま。
        assert_eq!(sam_full_jp.job_trait_total(JobTrait::StoreTp), 30);
    }

    #[test]
    fn test_blu_unlearned_trait_not_granted_by_gift() {
        // BLU が習得しない特性 (例: WAR の Smite, DRG の Strafe) はギフト適用外。
        // BLU は Smite/Strafe を持たない → 0 のまま。
        let chara1200 = build_blu99_with_jp(1200);
        assert_eq!(chara1200.job_trait_total(JobTrait::Smite), 0);
        assert_eq!(chara1200.job_trait_total(JobTrait::Strafe), 0);
    }

    #[test]
    fn test_blu_auto_regen_with_gift() {
        // BLU は AutoRegen を Lv16 から習得 (rank 1 = +1/3sec)。
        // 1200 JP で「ジョブ特性効果アップ」rank+2 → rank 3 = +3/3sec
        let chara0 = build_blu99_with_jp(0);
        let chara100 = build_blu99_with_jp(100);
        let chara1200 = build_blu99_with_jp(1200);
        assert_eq!(chara0.job_trait_total(JobTrait::AutoRegen), 1);
        assert_eq!(chara100.job_trait_total(JobTrait::AutoRegen), 2);
        assert_eq!(chara1200.job_trait_total(JobTrait::AutoRegen), 3);
    }
}
