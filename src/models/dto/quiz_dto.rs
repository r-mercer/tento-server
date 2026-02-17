use async_graphql::InputObject;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::errors::AppError;
use crate::models::domain::quiz::QuizStatus;
use crate::models::domain::quiz_question::{QuizQuestionOption, QuizQuestionType};
use crate::models::domain::{Quiz, QuizQuestion};

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
pub struct QuizQuestionDto {
	pub id: String,
	pub title: String,
	pub description: String,
	pub question_type: QuizQuestionType,
	pub options: Vec<QuizQuestionOption>,
	pub option_count: i16,
	pub order: i16,
	pub attempt_limit: i16,
	pub topic: String,
	pub created_at: DateTime<Utc>,
	pub modified_at: DateTime<Utc>,
}

impl From<QuizQuestion> for QuizQuestionDto {
	fn from(question: QuizQuestion) -> Self {
		let now = Utc::now();
		QuizQuestionDto {
			id: question.id,
			title: question.title,
			description: question.description,
			question_type: question.question_type,
			options: question.options,
			option_count: question.option_count,
			order: question.order,
			attempt_limit: question.attempt_limit,
			topic: question.topic,
			created_at: question.created_at.unwrap_or(now),
			modified_at: question.modified_at.unwrap_or(now),
		}
	}
}

impl TryFrom<QuizQuestionDto> for QuizQuestion {
	type Error = AppError;

	fn try_from(dto: QuizQuestionDto) -> Result<Self, Self::Error> {
		Ok(QuizQuestion {
			id: dto.id,
			title: dto.title,
			description: dto.description,
			question_type: dto.question_type,
			options: dto.options,
			option_count: dto.option_count,
			order: dto.order,
			attempt_limit: dto.attempt_limit,
			topic: dto.topic,
			created_at: Some(dto.created_at),
			modified_at: Some(dto.modified_at),
		})
	}
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
pub struct QuizDto {
	pub id: String,
	pub name: String,
	pub created_by_user_id: String,
	pub title: String,
	pub description: String,
	pub question_count: i16,
	pub required_score: i16,
	pub attempt_limit: i16,
	pub topic: String,
	pub status: QuizStatus,
	pub questions: Vec<QuizQuestionDto>,
	pub url: String,
	pub created_at: DateTime<Utc>,
	pub modified_at: DateTime<Utc>,
}

impl From<Quiz> for QuizDto {
	fn from(quiz: Quiz) -> Self {
		let now = Utc::now();
		QuizDto {
			id: quiz.id,
			name: quiz.name,
			created_by_user_id: quiz.created_by_user_id,
			title: quiz.title.unwrap_or_default(),
			description: quiz.description.unwrap_or_default(),
			question_count: quiz.question_count,
			required_score: quiz.required_score,
			attempt_limit: quiz.attempt_limit,
			topic: quiz.topic.unwrap_or_default(),
			status: quiz.status,
			questions: quiz
				.questions
				.unwrap_or_default()
				.into_iter()
				.map(QuizQuestionDto::from)
				.collect(),
			url: quiz.url,
			created_at: quiz.created_at.unwrap_or(now),
			modified_at: quiz.modified_at.unwrap_or(now),
		}
	}
}

impl TryFrom<QuizDto> for Quiz {
	type Error = AppError;

	fn try_from(dto: QuizDto) -> Result<Self, Self::Error> {
		let questions = if dto.questions.is_empty() {
			None
		} else {
			Some(
				dto.questions
					.into_iter()
					.map(QuizQuestion::try_from)
					.collect::<Result<Vec<_>, AppError>>()?,
			)
		};

		Ok(Quiz {
			id: dto.id,
			name: dto.name,
			created_by_user_id: dto.created_by_user_id,
			title: Some(dto.title),
			description: Some(dto.description),
			question_count: dto.question_count,
			required_score: dto.required_score,
			attempt_limit: dto.attempt_limit,
			topic: Some(dto.topic),
			status: dto.status,
			questions,
			url: dto.url,
			created_at: Some(dto.created_at),
			modified_at: Some(dto.modified_at),
		})
	}
}
