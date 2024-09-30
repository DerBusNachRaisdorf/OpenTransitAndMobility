use model::calendar::CalendarDate;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "service_exception_type", rename_all = "snake_case")]
pub enum ServiceExceptionType {
    Added,
    Removed,
}

impl Into<model::calendar::ServiceExceptionType> for ServiceExceptionType {
    fn into(self) -> model::calendar::ServiceExceptionType {
        match self {
            Self::Added => model::calendar::ServiceExceptionType::Added,
            Self::Removed => model::calendar::ServiceExceptionType::Removed,
        }
    }
}

impl From<model::calendar::ServiceExceptionType> for ServiceExceptionType {
    fn from(value: model::calendar::ServiceExceptionType) -> Self {
        match value {
            model::calendar::ServiceExceptionType::Added => Self::Added,
            model::calendar::ServiceExceptionType::Removed => Self::Removed,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct CalendarDateRow {
    pub service_id: Option<i32>,
    pub date: chrono::NaiveDate,
    pub exception_type: ServiceExceptionType,
}

impl CalendarDateRow {
    pub fn to_model(self) -> CalendarDate {
        CalendarDate {
            date: self.date,
            exception_type: self.exception_type.into(),
        }
    }

    pub fn from_model(calendar_date: CalendarDate) -> Self {
        Self {
            service_id: None,
            date: calendar_date.date,
            exception_type: calendar_date.exception_type.into(),
        }
    }
}
