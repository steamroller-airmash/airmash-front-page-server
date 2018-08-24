use serde::{Serializer};
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

#[derive(Deserialize, Clone, Debug)]
pub struct AltServerSpec {
	#[serde(rename = "type")]
	pub ty: u8,
	pub id: String,
	pub name: String,
	#[serde(rename = "nameShort")]
	pub name_short: String,
	pub players: u32,
	pub host: String
}

#[derive(Deserialize, Clone, Debug)]
pub struct AltRegionSpec {
	pub name: String,
	pub id: String,
	pub games: Vec<AltServerSpec>
}

#[derive(Deserialize, Clone, Debug)]
pub struct AltGameSpec {
	pub protocol: u8,
	pub country: String,
	pub data: String
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

impl AltServerSpec {
	pub fn into_normal(self) -> ServerSpec {
		ServerSpec {
			url: "https://game-".to_owned() + &self.host + ".airma.sh/" + &self.id,
			id: self.id,
			ty: self.ty,
			name: self.name,
			name_short: self.name_short,
			players: Some(self.players),
			host: self.host,
		}
	}
}

impl AltRegionSpec {
	pub fn into_normal(self) -> RegionSpec {
		RegionSpec {
			games: self.games
				.into_iter()
				.map(|x| x.into_normal())
				.collect(),
			id: self.id,
			name: self.name
		}
	}
}
