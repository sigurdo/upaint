use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum OptionUntagged<T> {
    #[default]
    None,
    #[serde(untagged)]
    Some(T),
}

impl<T> std::convert::From<OptionUntagged<T>> for Option<T> {
    fn from(value: OptionUntagged<T>) -> Self {
        match value {
            OptionUntagged::Some(t) => Some(t),
            OptionUntagged::None => None,
        }
    }
}

impl<T> std::convert::From<Option<T>> for OptionUntagged<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(t) => Self::Some(t),
            None => Self::None,
        }
    }
}

impl<'a, T> std::convert::From<&'a Option<T>> for OptionUntagged<&'a T> {
    fn from(value: &'a Option<T>) -> Self {
        match value {
            Some(t) => Self::Some(t),
            None => Self::None,
        }
    }
}

pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    Ok(OptionUntagged::deserialize(deserializer)
        .map_err(serde::de::Error::custom)?
        .into())
}

pub fn serialize<S: Serializer, T: Serialize + Clone>(
    value: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    OptionUntagged::serialize(&value.into(), serializer).map_err(serde::ser::Error::custom)
}
