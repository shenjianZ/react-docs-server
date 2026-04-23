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

    pub async fn status(
        &self,
        slug: String,
        lang: String,
        user_id: Option<String>,
        ip_address: Option<String>,
    ) -> Result<Option<page_feedbacks::Model>> {
        self.repo
            .find_by_subject(user_id.as_deref(), &slug, &lang, ip_address.as_deref())
            .await
    }

    pub async fn create(
        &self,
        payload: CreateFeedbackRequest,
        user_id: Option<String>,
        ip_address: Option<String>,
    ) -> Result<page_feedbacks::Model> {
        let CreateFeedbackRequest {
            value,
            comment,
            lang,
            slug,
        } = payload;

        let page_slug = slug.unwrap_or_else(|| "index".to_string());
        let page_lang = lang.unwrap_or_else(|| "zh-cn".to_string());

        if value != "helpful" && value != "unhelpful" {
            return Err(anyhow::anyhow!("反馈值无效"));
        }

        if let Some(model) = self
            .repo
            .find_by_subject(user_id.as_deref(), &page_slug, &page_lang, ip_address.as_deref())
            .await?
        {
            return Ok(model);
        }

        let active_model = FeedbackRepository::active_model(
            Uuid::new_v4().to_string(),
            page_slug,
            page_lang,
            value,
            comment,
            user_id,
            ip_address,
        );

        self.repo.insert(active_model).await
    }
}
