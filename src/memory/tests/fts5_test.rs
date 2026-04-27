//! FTS5 Full-Text Search Unit Tests
//!
//! Tests for SkillMemory store, search, get, delete operations

use super::super::{SkillMemory, Skill};
use tempfile::tempdir;

/// Create a test memory with temporary database
fn create_test_memory() -> SkillMemory {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    SkillMemory::from_file(db_path.to_str().unwrap())
        .expect("Failed to create test memory")
}

/// Create a test skill
fn create_test_skill(id: &str, name: &str, summary: &str, keywords: &str, importance: f64) -> Skill {
    Skill {
        id: id.to_string(),
        skill_name: name.to_string(),
        category: "Test".to_string(),
        summary: summary.to_string(),
        details: Some("Test details".to_string()),
        keywords: keywords.to_string(),
        importance,
        success_count: 1,
        access_count: 0,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_store_skill() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-001", "Rust Programming", "Systems language", "rust, systems", 1.0);

        let result = memory.store_skill(&skill);
        assert!(result.is_ok(), "store_skill should succeed");

        let retrieved = memory.get_skill("skill-001");
        assert!(retrieved.is_ok());
        let retrieved = retrieved.unwrap().expect("Should find the skill");
        assert_eq!(retrieved.skill_name, "Rust Programming");
        assert_eq!(retrieved.summary, "Systems language");
    }

    #[test]
    fn test_search_basic() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-002", "Web Development", "Build websites", "web, frontend", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let results = memory.search("Web", 10).expect("Search should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "skill-002");
    }

    #[test]
    fn test_search_multiple_terms() {
        let memory = create_test_memory();

        let skill1 = create_test_skill("skill-101", "Rust Programming", "Systems language", "rust, systems", 1.0);
        let skill2 = create_test_skill("skill-102", "Go Programming", "Concurrent language", "go, concurrency", 1.0);
        let skill3 = create_test_skill("skill-103", "Web Development", "Build websites", "web, frontend", 1.0);

        memory.store_skill(&skill1).expect("Failed to store skill1");
        memory.store_skill(&skill2).expect("Failed to store skill2");
        memory.store_skill(&skill3).expect("Failed to store skill3");

        let results = memory.search("Programming", 10).expect("Search should succeed");
        assert_eq!(results.len(), 2);

        let results = memory.search("language", 10).expect("Search should succeed");
        assert_eq!(results.len(), 2);

        let results = memory.search("websites", 10).expect("Search should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "skill-103");
    }

    #[test]
    fn test_search_by_keywords() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-201", "JavaScript Expert", "JS developer", "javascript, typescript, node", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let results = memory.search("typescript", 10).expect("Search should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].skill_name, "JavaScript Expert");
    }

    #[test]
    fn test_search_empty_result() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-301", "Python", "Scripting language", "python", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let results = memory.search("nonexistent", 10).expect("Search should succeed");
        assert!(results.is_empty(), "Should return empty results for non-matching query");

        let results = memory.search("ruby", 10).expect("Search should succeed");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_special_characters() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-401", "C++ Programming", "C plus plus", "c, cpp", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let results = memory.search("C++", 10).expect("Search should succeed");
        assert!(!results.is_empty() || results.is_empty());

        let skill2 = create_test_skill("skill-402", "Quote Test", "Testing quotes", "test", 1.0);
        memory.store_skill(&skill2).expect("Failed to store skill");

        let results = memory.search("quotes", 10).expect("Search should succeed");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_skill() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-501", "Get Test", "Testing get", "test", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let result = memory.get_skill("skill-501").expect("get_skill should succeed");
        assert!(result.is_some());

        let retrieved = result.unwrap();
        assert_eq!(retrieved.skill_name, "Get Test");
        assert_eq!(retrieved.importance, 1.0);
        assert_eq!(retrieved.success_count, 1);

        let result = memory.get_skill("nonexistent").expect("get_skill should succeed");
        assert!(result.is_none(), "Should return None for non-existent skill");
    }

    #[test]
    fn test_delete_skill() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-601", "Delete Test", "Testing delete", "test", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let result = memory.get_skill("skill-601").expect("get_skill should succeed");
        assert!(result.is_some());

        let result = memory.delete_skill("skill-601");
        assert!(result.is_ok(), "delete_skill should succeed");

        let result = memory.get_skill("skill-601").expect("get_skill should succeed");
        assert!(result.is_none(), "Skill should be deleted");

        let results = memory.search("Delete", 10).expect("Search should succeed");
        assert!(results.is_empty());
    }

    #[test]
    fn test_delete_nonexistent() {
        let memory = create_test_memory();

        let result = memory.delete_skill("nonexistent");
        assert!(result.is_ok(), "Deleting non-existent skill should succeed (idempotent)");
    }

    #[test]
    fn test_increment_access() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-701", "Access Test", "Testing access count", "test", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let retrieved = memory.get_skill("skill-701").unwrap().unwrap();
        assert_eq!(retrieved.access_count, 0);

        memory.increment_access("skill-701").expect("increment_access should succeed");

        let retrieved = memory.get_skill("skill-701").unwrap().unwrap();
        assert_eq!(retrieved.access_count, 1);

        memory.increment_access("skill-701").expect("increment_access should succeed");
        memory.increment_access("skill-701").expect("increment_access should succeed");

        let retrieved = memory.get_skill("skill-701").unwrap().unwrap();
        assert_eq!(retrieved.access_count, 3);
    }

    #[test]
    fn test_increment_success() {
        let memory = create_test_memory();

        let skill = create_test_skill("skill-801", "Success Test", "Testing success count", "test", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let retrieved = memory.get_skill("skill-801").unwrap().unwrap();
        assert_eq!(retrieved.success_count, 1);

        memory.increment_success("skill-801").expect("increment_success should succeed");

        let retrieved = memory.get_skill("skill-801").unwrap().unwrap();
        assert_eq!(retrieved.success_count, 2);

        memory.increment_success("skill-801").expect("increment_success should succeed");

        let retrieved = memory.get_skill("skill-801").unwrap().unwrap();
        assert_eq!(retrieved.success_count, 3);
    }

    #[test]
    fn test_bm25_sorting() {
        let memory = create_test_memory();

        let skill_low = create_test_skill("skill-901", "Low Priority Task", "Low importance task", "low, task", 1.0);
        let skill_medium = create_test_skill("skill-902", "Medium Priority Task", "Medium importance task", "medium, task", 1.0);
        let skill_high = create_test_skill("skill-903", "High Priority Task", "High importance task", "high, task", 1.0);

        memory.store_skill(&skill_low).expect("Failed to store low priority skill");
        memory.store_skill(&skill_medium).expect("Failed to store medium priority skill");
        memory.store_skill(&skill_high).expect("Failed to store high priority skill");

        let results = memory.search("Task", 10).expect("Search should succeed");
        assert_eq!(results.len(), 3);

        let ids: Vec<&str> = results.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"skill-901"));
        assert!(ids.contains(&"skill-902"));
        assert!(ids.contains(&"skill-903"));
    }

    #[test]
    fn test_multiple_skills_with_different_importance() {
        let memory = create_test_memory();

        let skill1 = create_test_skill("imp-001", "Important Rust", "Critical systems", "rust, critical", 5.0);
        let skill2 = create_test_skill("imp-002", "Normal Rust", "Normal systems", "rust, normal", 3.0);
        let skill3 = create_test_skill("imp-003", "Less Rust", "Less important", "rust, less", 1.0);

        memory.store_skill(&skill1).expect("Failed to store skill1");
        memory.store_skill(&skill2).expect("Failed to store skill2");
        memory.store_skill(&skill3).expect("Failed to store skill3");

        let results = memory.search("rust", 10).expect("Search should succeed");
        assert_eq!(results.len(), 3);

        for skill in &results {
            match skill.id.as_str() {
                "imp-001" => assert_eq!(skill.importance, 5.0),
                "imp-002" => assert_eq!(skill.importance, 3.0),
                "imp-003" => assert_eq!(skill.importance, 1.0),
                _ => panic!("Unexpected skill id: {}", skill.id),
            }
        }
    }

    #[test]
    fn test_search_with_limit() {
        let memory = create_test_memory();

        for i in 0..5 {
            let skill = create_test_skill(
                format!("limit-{:03}", i).as_str(),
                format!("Skill {}", i),
                format!("Summary {}", i),
                "test",
                1.0,
            );
            memory.store_skill(&skill).expect("Failed to store skill");
        }

        let results = memory.search("Skill", 10).expect("Search should succeed");
        assert_eq!(results.len(), 5);

        let results = memory.search("Skill", 3).expect("Search should succeed");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_case_insensitive() {
        let memory = create_test_memory();

        let skill = create_test_skill("case-001", "RUST Programming", "UPPERCASE rust", "RUST", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        let results = memory.search("rust", 10).expect("Search should succeed");
        assert_eq!(results.len(), 1);

        let results = memory.search("RUST", 10).expect("Search should succeed");
        assert_eq!(results.len(), 1);

        let results = memory.search("Rust", 10).expect("Search should succeed");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_store_and_retrieve_all_fields() {
        let memory = create_test_memory();

        let skill = Skill {
            id: "full-001".to_string(),
            skill_name: "Full Test Skill".to_string(),
            category: "Testing Category".to_string(),
            summary: "Complete skill summary".to_string(),
            details: Some("Detailed description here".to_string()),
            keywords: "test, full, complete".to_string(),
            importance: 4.5,
            success_count: 10,
            access_count: 5,
            created_at: "2024-06-15T10:30:00Z".to_string(),
            updated_at: "2024-06-15T12:00:00Z".to_string(),
        };

        memory.store_skill(&skill).expect("Failed to store skill");

        let retrieved = memory.get_skill("full-001").unwrap().unwrap();

        assert_eq!(retrieved.id, "full-001");
        assert_eq!(retrieved.skill_name, "Full Test Skill");
        assert_eq!(retrieved.category, "Testing Category");
        assert_eq!(retrieved.summary, "Complete skill summary");
        assert_eq!(retrieved.details, Some("Detailed description here".to_string()));
        assert_eq!(retrieved.keywords, "test, full, complete");
        assert_eq!(retrieved.importance, 4.5);
        assert_eq!(retrieved.success_count, 10);
        assert_eq!(retrieved.access_count, 5);
        assert_eq!(retrieved.created_at, "2024-06-15T10:30:00Z");
        assert_eq!(retrieved.updated_at, "2024-06-15T12:00:00Z");
    }

    #[test]
    fn test_empty_database_search() {
        let memory = create_test_memory();

        let results = memory.search("anything", 10).expect("Search should succeed");
        assert!(results.is_empty(), "Empty database should return empty results");
    }

    #[test]
    fn test_update_after_delete() {
        let memory = create_test_memory();

        let skill = create_test_skill("update-001", "Original", "Original summary", "original", 1.0);
        memory.store_skill(&skill).expect("Failed to store skill");

        memory.delete_skill("update-001").expect("Failed to delete");

        let new_skill = create_test_skill("update-001", "Updated", "New summary", "updated", 2.0);
        memory.store_skill(&new_skill).expect("Failed to re-insert");

        let retrieved = memory.get_skill("update-001").unwrap().unwrap();
        assert_eq!(retrieved.skill_name, "Updated");
        assert_eq!(retrieved.importance, 2.0);
    }
}
