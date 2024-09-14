use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize , Serialize)]
pub struct Search
{
    pub name: Option<String>,
    pub f: Option<f32>,
    pub t: Option<f32>,
    pub mark: Option<String>,
    pub p: Option<i32>,
    pub l: Option<i32>
}
