use serde::{Deserialize, Serialize};


#[derive(Debug,Serialize , Deserialize)]
pub struct InvoiceIdReturn {
    pub inv_num: i32,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct Search {
    pub cst_name: Option<String>,
    pub y : i32,
    pub n : Option<i32>,
    pub p: Option<i16>,
    pub l: Option<i16>,
}