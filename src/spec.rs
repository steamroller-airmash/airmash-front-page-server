use serde::Serializer;
use serde_json;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerSpec {
	pub url: String,
	#[serde(rename = "type")]
	pub ty: u8,
	pub id: String,
	pub name: String,
	#[serde(rename = "nameShort")]
	pub name_short: String,
	#[serde(skip_deserializing)]
	pub players: Option<u32>,
	pub host: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegionSpec {
	pub name: String,
	pub id: String,
	pub games: Vec<ServerSpec>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSpec {
	pub protocol: u8,
	#[serde(skip_deserializing)]
	pub country: String,
	#[serde(serialize_with = "regionspec_serialize")]
	pub data: Vec<RegionSpec>,
}

fn regionspec_serialize<S>(obj: &Vec<RegionSpec>, ser: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let serialized = serde_json::to_string(obj).unwrap();
	ser.serialize_str(&serialized)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerResponse {
	pub players: u32,
}
