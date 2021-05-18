#[derive(Serialize, Deserialize, Debug)]
struct PgnDefinition {
    #[serde(rename = "PGN")]
    pub pgn: u32,
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Complete")]
    pub complete: bool,
    #[serde(rename = "Length")]
    pub length: u32,
    #[serde(rename = "RepeatingFields", default)]
    pub repeating_fields: u32,
    #[serde(rename = "Fields")]
    pub fields: Vec<PgnDefinitionField>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PgnDefinitionField {
    #[serde(rename = "Id")]
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PgnsFile {
    #[serde(rename = "Comment")]
    pub comment: String,
    #[serde(rename = "CreatorCode")]
    pub creator_code: String,
    #[serde(rename = "License")]
    pub license: String,
    #[serde(rename = "PGNs")]
    pub pgns: Pgns,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pgns {
    #[serde(rename = "PGNInfo")]
    pub pgn_infos: Vec<PgnInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PgnInfo {
    #[serde(rename = "PGN")]
    pub pgn: u32,
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Length")]
    pub length: usize,
    #[serde(rename = "Type")]
    pub xtype: String,
    #[serde(rename = "Fields", default)]
    pub fields: Fields,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fields {
    #[serde(rename = "Field", default)]
    pub fields: Vec<Field>,
}

impl std::default::Default for Fields {
    fn default() -> Fields {
        Fields { fields: vec![] }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    #[serde(rename = "Order")]
    pub order: String,
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Signed")]
    pub signed: bool,
    #[serde(rename = "BitLength")]
    pub bit_length: usize,
    #[serde(rename = "BitOffset", default)]
    pub bit_offset: usize,
    #[serde(rename = "Type", default)]
    pub n2k_type: String,
    #[serde(rename = "Resolution", default)]
    pub resolution: f32,
    #[serde(rename = "EnumValues", default)]
    pub enum_values: EnumValues,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnumValues {
    #[serde(rename = "EnumPair", default)]
    pub enum_values: Vec<EnumPair>,
}

impl std::default::Default for EnumValues {
    fn default() -> EnumValues {
        EnumValues {
            enum_values: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnumPair {
    #[serde(rename = "Value")]
    pub value: String,
    #[serde(rename = "Name")]
    pub name: String,
}
