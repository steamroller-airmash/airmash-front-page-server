use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_json;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(remote = "ServerSpec")]
pub struct ServerSpec {
	#[serde(getter = "ServerSpec::url")]
	#[serde(default)]
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
	pub path: String
}

impl Serialize for ServerSpec {
	fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
		ServerSpec::serialize(self, ser)
	}
}

impl<'de> Deserialize<'de> for ServerSpec {
	fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
		ServerSpec::deserialize(de)
	}
}

impl ServerSpec {
	pub fn url(&self) -> String {
		if self.url == "" {
			self.host.clone() + "/" + &self.path
		} else {
			self.url.clone()
		}
	}
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
	pub host: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AltRegionSpec {
	pub name: String,
	pub id: String,
	pub games: Vec<AltServerSpec>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AltGameSpec {
	pub protocol: u8,
	pub country: String,
	pub data: String,
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
