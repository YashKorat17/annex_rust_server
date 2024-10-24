use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize ,Deserialize)]
pub struct Customer {
    pub _id: Option<String>,
    pub b_name: String,
    pub name: String,
    pub city: String,
    pub state: String,
    pub op_bal: f32,
    pub op_fine: f32,
    pub ph: Option<Vec<i64>>,
    pub email: String,
    pub gstin: Option<String>,
    pub pan: Option<String>,
    pub logo: Option<String>,
    pub anx_id: Option<String>,
    pub gr : Option<String>,
    pub t : Option<String>,
    pub max_fine: Option<f32>,
    pub max_bal: Option<f32>,
    pub is_act: bool
}

#[derive(Debug, Deserialize)]
pub struct AnnexIdCheckUsername
{
    pub u_name: Option<String>,
}


#[derive(Debug, Deserialize)]
pub struct AnnexIdCheckGstin
{
    pub gstin: Option<String>,
}


#[derive(Debug, Serialize , Deserialize)]
pub struct Users
{
    pub _id: String,
    pub username: Option<String>,
    pub gstin: Option<String>,
    pub state: String,
    pub city: String,
    pub b_name: String,
    pub name: String,
    pub logo : Option<String>
}

#[derive(Debug, Serialize)]
pub struct AnnexResponse
{
    pub _id: String,
    pub username: String,
    pub gstin: Option<String>,
    pub msg: String,
    pub state: String,
    pub city: String,
    pub b_name: String,
    pub name: String,
    pub logo : Option<String>
}


#[derive(Debug, Deserialize)]
pub struct GetInvoices
{
    pub anx_id: String,
}

#[derive(Debug, Deserialize , Serialize)]
pub struct Search
{
    pub cst_name: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub p: Option<i32>,
    pub l: Option<i32>
}

#[derive(Debug, Deserialize)]
pub struct StatementId
{
    pub id: String,
}