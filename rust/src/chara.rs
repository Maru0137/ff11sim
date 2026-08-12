use std::option::Option;

use crate::gift::Gift;
use crate::job::{Job, JobTrait};
use crate::job_points::JobPointCategories;
use crate::race::Race;
use crate::skills::CharacterSkills;
use crate::status::{BonusStats, MeritPoints, StatusKind, calc_master_lv_bonus, calc_status};

/// `Chara::status()` のソース別分解。
/// race / main_job / support_job はゲーム仕様上「合算してから切り捨て」のため
/// f32 のまま保持する (個別に floor すると合計が合わない)。
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusParts {
    pub race: f32,
    pub main_job: f32,
    pub support_job: f32,
    pub mlv: i32,
    pub merit: i32,
    /// HP/MP 特性 (MaxHpBoost 等) のうちメインジョブ採用分
    pub trait_main: i32,
    /// HP/MP 特性のうちサポートジョブ採用分
    pub trait_support: i32,
    pub equip: i32,
}

impl StatusParts {
    pub fn total(&self) -> i32 {
        (self.race + self.main_job + self.support_job).floor() as i32
            + self.mlv
            + self.merit
            + self.equip
            + self.trait_main
            + self.trait_support
    }
}

/// ジョブ特性のメイン/サポート別の値。採用規則 (絶対値の大きい方、同値はメイン)
/// は adopted_* に集約する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraitBreakdown {
    pub main: i32,
    pub support: i32,
}

impl TraitBreakdown {
    fn main_wins(&self) -> bool {
        self.main.abs() >= self.support.abs()
    }

    /// 採用された値
    pub fn adopted(&self) -> i32 {
        if self.main_wins() {
            self.main
        } else {
            self.support
        }
    }

    /// 採用値のうちメインジョブ分 (サポート採用時は 0)
    pub fn adopted_main(&self) -> i32 {
        if self.main_wins() { self.main } else { 0 }
    }

    /// 採用値のうちサポートジョブ分 (メイン採用時は 0)
    pub fn adopted_support(&self) -> i32 {
        if self.main_wins() { 0 } else { self.support }
    }
}

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
        self.status_parts(kind).total()
    }

    /// ステータス値のソース別分解。`status()` はこの合成 (`StatusParts::total`)。
    pub fn status_parts(&self, kind: StatusKind) -> StatusParts {
        // For MP: if main job has no MP, all sources contribute 0 (equip included)
        if kind == StatusKind::Mp && self.main_job.status_grade(StatusKind::Mp).is_none() {
            return StatusParts::default();
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

        // Job trait bonus for HP/MP (採用ジョブ側の行に振り分ける)
        let (trait_main, trait_support) = match kind {
            StatusKind::Hp => {
                let b1 = self.job_trait_breakdown(JobTrait::MaxHpBoost);
                let b2 = self.job_trait_breakdown(JobTrait::MaxHpBoost2);
                (
                    b1.adopted_main() + b2.adopted_main(),
                    b1.adopted_support() + b2.adopted_support(),
                )
            }
            StatusKind::Mp => {
                let b = self.job_trait_breakdown(JobTrait::MaxMpBoost);
                (b.adopted_main(), b.adopted_support())
            }
            _ => (0, 0),
        };

        StatusParts {
            race: status_race,
            main_job: status_main_job,
            support_job: status_support_job,
            mlv: calc_master_lv_bonus(kind, self.master_lv),
            merit: self.merit_points.status_bonus(kind),
            trait_main,
            trait_support,
            equip: self.bonus_stats.get(kind),
        }
    }

    /// Calculate total job trait bonus from main + support job.
    /// メインジョブが BLU の場合、ギフト「ジョブ特性効果アップ」(100JP=+1, 1200JP=+2 ランク)
    /// を base rank に加算する (除外特性: Gilfinder/DoubleAttack/AutoRefresh/TripleAttack)。
    ///
    /// 集約規則: 「効果の強い方」を採用。通常特性 (正値) は max、
    /// MartialArts のような負値特性 (隔短縮) は min を取りたいため、
    /// 単純に絶対値が大きい方を選ぶ (符号は同一前提)。
    pub fn job_trait_total(&self, trait_kind: JobTrait) -> i32 {
        self.job_trait_breakdown(trait_kind).adopted()
    }

    /// ジョブ特性のメイン/サポート別の値。採用は `TraitBreakdown::adopted*`。
    pub fn job_trait_breakdown(&self, trait_kind: JobTrait) -> TraitBreakdown {
        let main = self.main_job_trait_bonus(trait_kind);
        let support = match (&self.support_job, &self.support_lv) {
            (Some(job), Some(lv)) => job.trait_bonus(trait_kind, *lv),
            _ => 0,
        };
        TraitBreakdown { main, support }
    }

    /// メインジョブ単独のジョブ特性ボーナス (BLU の JobTraitEffectUp ギフトを考慮)。
    fn main_job_trait_bonus(&self, trait_kind: JobTrait) -> i32 {
        let base_rank = self.main_job.trait_rank_at_lv(trait_kind, self.main_lv);
        if base_rank == 0 {
            // 未習得特性にはギフトのランクアップは適用されない
            return 0;
        }
        // BLU の「ジョブ特性効果アップ」ギフトで base rank を強化
        // (除外特性: Gilfinder/DoubleAttack/AutoRefresh/TripleAttack)
        let bonus_rank = if !trait_kind.is_blu_effect_up_excluded() {
            self.main_job
                .gift_value(Gift::JobTraitEffectUp, self.job_points.total_jp_spent())
                as usize
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
            (0..=50).contains(&master_lv),
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

    // 注: BLU の特性は青魔法セットによって決まるため、青魔法対応までは
    //     trait_levels に BLU の習得レベルを定義しない。
    //     そのため BLU 個別の特性 / ギフト適用テストは青魔法対応後に追加する。

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
        let chara1200 = build_blu99_with_jp(1200);
        assert_eq!(chara1200.job_trait_total(JobTrait::Smite), 0);
        assert_eq!(chara1200.job_trait_total(JobTrait::Strafe), 0);
    }
}
