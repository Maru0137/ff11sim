use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::chara::Chara;
use crate::job::Job;
use crate::race::Race;
use crate::status::MeritPoints;

/// ジョブごとのレベル情報
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct JobLevel {
    pub level: i32,
    pub master_lv: i32,
}

/// キャラクタープロファイル（名前・種族・全ジョブのレベル情報・メリットポイント）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub name: String,
    pub race: Race,
    pub job_levels: EnumMap<Job, JobLevel>,
    pub merit_points: MeritPoints,
}

impl CharacterProfile {
    pub fn new(name: String, race: Race) -> Self {
        Self {
            name,
            race,
            job_levels: EnumMap::default(),
            merit_points: MeritPoints::default(),
        }
    }

    pub fn set_job_level(&mut self, job: Job, level: i32, master_lv: i32) {
        assert!(
            level >= 0 && level <= 99,
            "level must be between 0 and 99"
        );
        assert!(
            master_lv >= 0 && master_lv <= 50,
            "master_lv must be between 0 and 50"
        );
        self.job_levels[job] = JobLevel { level, master_lv };
    }

    /// 指定したメインジョブ・サポートジョブ構成で Chara を生成する。
    /// サポートジョブの有効レベルは min(実レベル, メインLv/2 + マスターLv/5) で自動計算。
    pub fn to_chara(&self, main_job: Job, support_job: Option<Job>) -> Result<Chara, String> {
        let main_jl = &self.job_levels[main_job];
        if main_jl.level == 0 {
            return Err(format!("{:?} is not leveled", main_job));
        }

        let mut builder = Chara::builder()
            .race(self.race)
            .main_job(main_job, main_jl.level)
            .master_lv(main_jl.master_lv)
            .merit_points(self.merit_points);

        if let Some(sub) = support_job {
            let sub_jl = &self.job_levels[sub];
            if sub_jl.level == 0 {
                return Err(format!("Support job {:?} is not leveled", sub));
            }
            let cap = main_jl.level / 2 + main_jl.master_lv / 5;
            let effective_lv = std::cmp::min(sub_jl.level, cap);
            if effective_lv > 0 {
                builder = builder.support_job(sub, effective_lv);
            }
        }

        builder.build().map_err(|e| e.to_string())
    }
}

/// キャラクター登録管理
pub struct CharaRegistry {
    characters: Vec<CharacterProfile>,
}

impl CharaRegistry {
    pub fn new() -> Self {
        Self {
            characters: Vec::new(),
        }
    }

    pub fn register(&mut self, profile: CharacterProfile) -> Result<(), String> {
        if self.characters.iter().any(|c| c.name == profile.name) {
            return Err(format!("Character '{}' already exists", profile.name));
        }
        self.characters.push(profile);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&CharacterProfile> {
        self.characters.iter().find(|c| c.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut CharacterProfile> {
        self.characters.iter_mut().find(|c| c.name == name)
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let len = self.characters.len();
        self.characters.retain(|c| c.name != name);
        self.characters.len() != len
    }

    pub fn list(&self) -> Vec<&str> {
        self.characters.iter().map(|c| c.name.as_str()).collect()
    }

    /// 登録済みキャラクターを指定して Chara を生成する
    pub fn to_chara(
        &self,
        name: &str,
        main_job: Job,
        support_job: Option<Job>,
    ) -> Result<Chara, String> {
        let profile = self
            .get(name)
            .ok_or_else(|| format!("Character '{}' not found", name))?;
        profile.to_chara(main_job, support_job)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::StatusKind;

    #[test]
    fn test_character_profile_new() {
        let profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        assert_eq!(profile.name, "TestChar");
        assert_eq!(profile.race, Race::Hum);
        // All job levels should be 0
        for (_job, jl) in &profile.job_levels {
            assert_eq!(jl.level, 0);
            assert_eq!(jl.master_lv, 0);
        }
    }

    #[test]
    fn test_set_job_level() {
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 50);
        assert_eq!(profile.job_levels[Job::War].level, 99);
        assert_eq!(profile.job_levels[Job::War].master_lv, 50);
        // Other jobs should still be 0
        assert_eq!(profile.job_levels[Job::Blm].level, 0);
    }

    #[test]
    fn test_to_chara_war_drg() {
        // Hum/War99/Drg/MLV50 — 既存テストと同じ結果になることを検証
        // Support calc lv = min(59, 99/2 + 50/5) = min(59, 49+10) = 59
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 50);
        profile.set_job_level(Job::Drg, 59, 0);

        let chara = profile.to_chara(Job::War, Some(Job::Drg)).unwrap();
        assert_eq!(chara.status(StatusKind::Hp), 1765);
        assert_eq!(chara.status(StatusKind::Str), 147);
        assert_eq!(chara.status(StatusKind::Mp), 0);
    }

    #[test]
    fn test_to_chara_cor_sam() {
        // Gal/Cor99/Sam/MLV50 — 既存テストと同じ結果になることを検証
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Gal);
        profile.set_job_level(Job::Cor, 99, 50);
        profile.set_job_level(Job::Sam, 59, 0);

        let chara = profile.to_chara(Job::Cor, Some(Job::Sam)).unwrap();
        assert_eq!(chara.status(StatusKind::Str), 138);
        assert_eq!(chara.status(StatusKind::Dex), 141);
        assert_eq!(chara.status(StatusKind::Vit), 143);
        assert_eq!(chara.status(StatusKind::Agi), 138);
        assert_eq!(chara.status(StatusKind::Int), 135);
        assert_eq!(chara.status(StatusKind::Mnd), 132);
        assert_eq!(chara.status(StatusKind::Chr), 127);
    }

    #[test]
    fn test_to_chara_blm_with_mp() {
        // Tar/Blm99/Rdm@59/MLV50
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Tar);
        profile.set_job_level(Job::Blm, 99, 50);
        profile.set_job_level(Job::Rdm, 59, 0);

        let chara = profile.to_chara(Job::Blm, Some(Job::Rdm)).unwrap();
        assert_eq!(chara.status(StatusKind::Mp), 1692);
    }

    #[test]
    fn test_to_chara_no_support_job() {
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 0);

        let chara = profile.to_chara(Job::War, None).unwrap();
        assert_eq!(chara.status(StatusKind::Hp), 1160);
        assert_eq!(chara.status(StatusKind::Str), 82);
    }

    #[test]
    fn test_to_chara_support_level_capped() {
        // サポートジョブの実レベルが上限より高い場合、キャップされることを検証
        // メインLv75, マスターLv0 -> キャップ = 75/2 + 0/5 = 37
        // サポートジョブの実レベルは99だが、37にキャップされる
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 75, 0);
        profile.set_job_level(Job::Drg, 99, 0);

        let chara = profile.to_chara(Job::War, Some(Job::Drg)).unwrap();
        assert_eq!(chara.support_lv, Some(37));
    }

    #[test]
    fn test_to_chara_support_level_uses_actual_when_lower() {
        // サポートジョブの実レベルが上限より低い場合、実レベルが使われることを検証
        // メインLv99, マスターLv50 -> キャップ = 99/2 + 50/5 = 49+10 = 59
        // サポートジョブの実レベルは30なので、30が使われる
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 50);
        profile.set_job_level(Job::Drg, 30, 0);

        let chara = profile.to_chara(Job::War, Some(Job::Drg)).unwrap();
        assert_eq!(chara.support_lv, Some(30));
    }

    #[test]
    fn test_to_chara_unleveled_main_job_error() {
        let profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        let result = profile.to_chara(Job::War, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not leveled"));
    }

    #[test]
    fn test_to_chara_unleveled_support_job_error() {
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 50);

        let result = profile.to_chara(Job::War, Some(Job::Drg));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not leveled"));
    }

    #[test]
    fn test_to_chara_with_merit_points() {
        let mut profile = CharacterProfile::new("TestChar".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 0);
        profile.merit_points = MeritPoints {
            hp: 5,
            mp: 0,
            str_: 3,
            dex: 0,
            vit: 0,
            agi: 0,
            int: 0,
            mnd: 0,
            chr: 0,
        };

        let chara = profile.to_chara(Job::War, None).unwrap();
        // HP = race(D:485) + job(B:675) + merit(5*10=50) = 1210
        assert_eq!(chara.status(StatusKind::Hp), 1210);
        // STR = race(D:37.5) + job(A:45) + merit(3*1=3) = 85
        assert_eq!(chara.status(StatusKind::Str), 85);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = CharaRegistry::new();
        let mut profile = CharacterProfile::new("Adventurer".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 50);

        registry.register(profile).unwrap();

        let retrieved = registry.get("Adventurer").unwrap();
        assert_eq!(retrieved.name, "Adventurer");
        assert_eq!(retrieved.race, Race::Hum);
        assert_eq!(retrieved.job_levels[Job::War].level, 99);
    }

    #[test]
    fn test_registry_duplicate_name_error() {
        let mut registry = CharaRegistry::new();
        registry
            .register(CharacterProfile::new("Adventurer".to_string(), Race::Hum))
            .unwrap();

        let result = registry.register(CharacterProfile::new("Adventurer".to_string(), Race::Elv));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = CharaRegistry::new();
        registry
            .register(CharacterProfile::new("Adventurer".to_string(), Race::Hum))
            .unwrap();

        assert!(registry.remove("Adventurer"));
        assert!(registry.get("Adventurer").is_none());
        assert!(!registry.remove("Adventurer")); // already removed
    }

    #[test]
    fn test_registry_list() {
        let mut registry = CharaRegistry::new();
        registry
            .register(CharacterProfile::new("Alice".to_string(), Race::Hum))
            .unwrap();
        registry
            .register(CharacterProfile::new("Bob".to_string(), Race::Elv))
            .unwrap();

        let names = registry.list();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Bob"));
    }

    #[test]
    fn test_registry_get_mut() {
        let mut registry = CharaRegistry::new();
        registry
            .register(CharacterProfile::new("Adventurer".to_string(), Race::Hum))
            .unwrap();

        let profile = registry.get_mut("Adventurer").unwrap();
        profile.set_job_level(Job::War, 99, 50);

        let retrieved = registry.get("Adventurer").unwrap();
        assert_eq!(retrieved.job_levels[Job::War].level, 99);
    }

    #[test]
    fn test_registry_to_chara() {
        let mut registry = CharaRegistry::new();
        let mut profile = CharacterProfile::new("Adventurer".to_string(), Race::Hum);
        profile.set_job_level(Job::War, 99, 50);
        profile.set_job_level(Job::Drg, 59, 0);
        registry.register(profile).unwrap();

        let chara = registry
            .to_chara("Adventurer", Job::War, Some(Job::Drg))
            .unwrap();
        assert_eq!(chara.status(StatusKind::Hp), 1765);
    }

    #[test]
    fn test_registry_to_chara_not_found() {
        let registry = CharaRegistry::new();
        let result = registry.to_chara("Unknown", Job::War, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
