use sea_orm::Order as SeaOrmOrder;
use serde::Serialize;
use typeshare::typeshare;

#[derive(Debug, Serialize, Default, Clone, FromFormField)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    #[default]
    Desc,
    Asc,
}

impl From<Direction> for SeaOrmOrder {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Desc => SeaOrmOrder::Desc,
            Direction::Asc => SeaOrmOrder::Asc,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default, FromForm)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Order<T>
where
    T: Default + Clone,
{
    pub by: Option<T>,
    pub direction: Option<Direction>,
}

impl<T> Order<T>
where
    T: Default + Clone,
{
    pub fn by(&self) -> T {
        self.by.clone().unwrap_or_default()
    }

    pub fn direction(&self) -> Direction {
        self.direction.clone().unwrap_or_default()
    }
}
