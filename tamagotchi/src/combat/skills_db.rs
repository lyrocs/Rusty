/// Skill database helpers
///
/// Provides methods to get skills by job class.

use super::skills::JrpgSkill;

impl JrpgSkill {
    /// Get skills for a specific job
    pub fn get_skills_for_job(job: &str) -> [JrpgSkill; 3] {
        match job {
            "Swordsman" => Self::get_swordsman_skills(),
            "Mage" => Self::get_mage_skills(),
            "Archer" => Self::get_archer_skills(),
            "Thief" => Self::get_thief_skills(),
            "Acolyte" => Self::get_acolyte_skills(),
            "Merchant" => Self::get_merchant_skills(),
            _ => Self::get_swordsman_skills(), // Default to Swordsman
        }
    }
}
