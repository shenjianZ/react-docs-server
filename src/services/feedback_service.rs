use crate::domain::dto::feedback::CreateFeedbackRequest;
use crate::domain::entities::page_feedbacks;
use crate::repositories::feedback_repository::FeedbackRepository;
use anyhow::Result;
use uuid::Uuid;

pub struct FeedbackService {
    repo: FeedbackRepository,
}

impl FeedbackService {
    pub fn new(repo: FeedbackRepository) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        payload: CreateFeedbackRequest,
        user_id: Option<String>,
    ) -> Result<page_feedbacks::Model> {
        if payload.value != "helpful" && payload.value != "unhelpful" {
            return Err(anyhow::anyhow!("反馈值无效"));
        }

        let active_model = FeedbackRepository::active_model(
            Uuid::new_v4().to_string(),
            payload.slug.unwrap_or_else(|| "index".to_string()),
            payload.lang.unwrap_or_else(|| "zh-cn".to_string()),
            payload.value,
            payload.comment,
            user_id,
        );

        self.repo.insert(active_model).await
    }
}
