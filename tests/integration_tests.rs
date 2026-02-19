use tento_server::models::domain::Quiz;
use tento_server::models::dto::quiz_dto::QuizDto;
use tento_server::models::dto::request::{QuizDraftDto, UpdateQuizInput};

#[test]
fn test_quiz_serialization() {
    let quiz = Quiz::test_quiz("Test Quiz", "user123");

    let json = serde_json::to_string(&quiz).unwrap();
    let deserialized: Quiz = serde_json::from_str(&json).unwrap();

    assert_eq!(quiz.id, deserialized.id);
    assert_eq!(quiz.name, deserialized.name);
    assert_eq!(quiz.created_by_user_id, deserialized.created_by_user_id);
    assert_eq!(quiz.status, deserialized.status);
}

#[test]
fn test_quiz_dto_conversion() {
    let quiz = Quiz::test_quiz_with_title("Test Quiz", "user123", "Quiz Title", "Quiz Description");

    let dto = QuizDto::from(quiz.clone());
    assert_eq!(dto.name, quiz.name);
    assert_eq!(dto.created_by_user_id, quiz.created_by_user_id);
    assert_eq!(dto.title, "Quiz Title");
    assert_eq!(dto.description, "Quiz Description");

    let back_to_quiz: Quiz = dto.try_into().unwrap();
    assert_eq!(back_to_quiz.name, quiz.name);
}

#[test]
fn test_quiz_draft_dto() {
    let draft = QuizDraftDto {
        name: "My Quiz".to_string(),
        question_count: 10,
        required_score: 70,
        attempt_limit: 3,
        url: "https://example.com/article".to_string(),
    };

    let json = serde_json::to_string(&draft).unwrap();
    let deserialized: QuizDraftDto = serde_json::from_str(&json).unwrap();

    assert_eq!(draft.name, deserialized.name);
    assert_eq!(draft.question_count, deserialized.question_count);
}

#[test]
fn test_update_quiz_input_serialization() {
    let input = UpdateQuizInput {
        id: "quiz-123".to_string(),
        title: Some("Updated Title".to_string()),
        description: Some("Updated Description".to_string()),
        questions: None,
    };

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: UpdateQuizInput = serde_json::from_str(&json).unwrap();

    assert_eq!(input.id, deserialized.id);
    assert_eq!(input.title, deserialized.title);
    assert_eq!(input.description, deserialized.description);
}

#[test]
fn test_update_quiz_partial_fields() {
    let input = UpdateQuizInput {
        id: "quiz-123".to_string(),
        title: Some("New Title".to_string()),
        description: None,
        questions: None,
    };

    assert!(input.title.is_some());
    assert!(input.description.is_none());
}

#[test]
fn test_quiz_default_values() {
    let quiz = Quiz::test_quiz("Default Quiz", "user1");

    assert_eq!(
        quiz.status,
        tento_server::models::domain::quiz::QuizStatus::Draft
    );
    assert_eq!(quiz.question_count, 5);
    assert_eq!(quiz.required_score, 70);
    assert_eq!(quiz.attempt_limit, 3);
    assert!(quiz.title.is_none());
    assert!(quiz.questions.is_none());
}

#[test]
fn test_quiz_id_format() {
    let quiz = Quiz::test_quiz("ID Test", "user1");

    let uuid_regex =
        regex::Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
            .unwrap();

    assert!(uuid_regex.is_match(&quiz.id));
}
