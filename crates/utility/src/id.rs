use std::{borrow::Cow, fmt, hash, marker::PhantomData};

use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Serialize};

pub trait Key {
    fn string_key(&self) -> String;
}

pub struct CompoundKey {
    keys: Vec<Box<dyn Key>>,
}

impl Key for CompoundKey {
    fn string_key(&self) -> String {
        let string_keys = self
            .keys
            .iter()
            .map(|key| key.string_key())
            .collect::<Vec<_>>()
            .join(",");
        format!("({})", string_keys)
    }
}

pub trait HasId {
    type IdType;
}

pub struct Id<T: HasId>(T::IdType, PhantomData<T>);

pub type IdString = String; // TODO das ist dumm und muss wieder weg.

impl<T: HasId> Id<T> {
    pub fn new(inner: T::IdType) -> Self {
        Self(inner, PhantomData)
    }
}

impl<T: HasId> Key for Id<T>
where
    T::IdType: fmt::Display,
{
    fn string_key(&self) -> String {
        format!("{}", self.0)
    }
}

impl<T: HasId> Id<T>
where
    T::IdType: Clone,
{
    pub fn raw(&self) -> T::IdType {
        self.0.clone()
    }

    pub fn raw_ref<'a, R>(&'a self) -> &'a R
    where
        T::IdType: AsRef<R>,
        R: ?Sized,
    {
        self.0.as_ref()
    }
}

pub trait IdWrapper<T: HasId>
where
    T::IdType: Clone,
{
    type ResultWrapper<R>;

    fn raw(self) -> Self::ResultWrapper<T::IdType>;
    fn raw_ref<'a, R>(&'a self) -> Self::ResultWrapper<&'a R>
    where
        R: ?Sized,
        T::IdType: AsRef<R>;
}

impl<T: HasId> IdWrapper<T> for Option<Id<T>>
where
    T::IdType: Clone,
{
    type ResultWrapper<R> = Option<R>;

    fn raw(self) -> Self::ResultWrapper<T::IdType> {
        self.map(|id| id.raw())
    }

    fn raw_ref<'a, R>(&'a self) -> Self::ResultWrapper<&'a R>
    where
        R: ?Sized,
        T::IdType: AsRef<R>,
    {
        self.as_ref().map(|id| id.raw_ref())
    }
}

impl<T: HasId> IdWrapper<T> for Option<&Id<T>>
where
    T::IdType: Clone,
{
    type ResultWrapper<R> = Option<R>;

    fn raw(self) -> Self::ResultWrapper<T::IdType> {
        self.map(|id| id.raw())
    }

    fn raw_ref<'a, R>(&'a self) -> Self::ResultWrapper<&'a R>
    where
        R: ?Sized,
        T::IdType: AsRef<R>,
    {
        self.as_ref().map(|id| id.raw_ref())
    }
}

impl<T: HasId> IdWrapper<T> for Vec<Id<T>>
where
    T::IdType: Clone,
{
    type ResultWrapper<R> = Vec<R>;

    fn raw(self) -> Self::ResultWrapper<T::IdType> {
        self.into_iter().map(|id| id.raw()).collect()
    }

    fn raw_ref<'a, R>(&'a self) -> Self::ResultWrapper<&'a R>
    where
        R: ?Sized,
        T::IdType: AsRef<R>,
    {
        self.iter().map(|id| id.raw_ref()).collect()
    }
}

impl<T: HasId> IdWrapper<T> for Vec<&Id<T>>
where
    T::IdType: Clone,
{
    type ResultWrapper<R> = Vec<R>;

    fn raw(self) -> Self::ResultWrapper<T::IdType> {
        self.into_iter().map(|id| id.raw()).collect()
    }

    fn raw_ref<'a, R>(&'a self) -> Self::ResultWrapper<&'a R>
    where
        R: ?Sized,
        T::IdType: AsRef<R>,
    {
        self.iter().map(|id| id.raw_ref()).collect()
    }
}

impl<T: HasId> IdWrapper<T> for &[Id<T>]
where
    T::IdType: Clone,
{
    type ResultWrapper<R> = Vec<R>;

    fn raw(self) -> Self::ResultWrapper<T::IdType> {
        self.into_iter().map(|id| id.raw()).collect()
    }

    fn raw_ref<'a, R>(&'a self) -> Self::ResultWrapper<&'a R>
    where
        R: ?Sized,
        T::IdType: AsRef<R>,
    {
        self.iter().map(|id| id.raw_ref()).collect()
    }
}

impl<T: HasId> IdWrapper<T> for &[&Id<T>]
where
    T::IdType: Clone,
{
    type ResultWrapper<R> = Vec<R>;

    fn raw(self) -> Self::ResultWrapper<T::IdType> {
        self.into_iter().map(|id| id.raw()).collect()
    }

    fn raw_ref<'a, R>(&'a self) -> Self::ResultWrapper<&'a R>
    where
        R: ?Sized,
        T::IdType: AsRef<R>,
    {
        self.iter().map(|id| id.raw_ref()).collect()
    }
}

impl<T: HasId> Id<T>
where
    T::IdType: From<String>,
{
    pub fn from_name(name: &str) -> Self {
        // TODO: efficiency concerns
        let inner = name
            .chars()
            .map(|c| match c {
                ' ' => "-".to_owned(),
                'ä' => "ae".to_owned(),
                'ö' => "oe".to_owned(),
                'ü' => "ue".to_owned(),
                'ß' => "ss".to_owned(),
                ch if ch.is_ascii() => ch.to_string(),
                _ => "".to_owned(),
            })
            .collect::<Vec<String>>()
            .join("");
        Self::new(inner.into())
    }
}

impl<T: HasId> Default for Id<T>
where
    T::IdType: Clone + Default,
{
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

impl<T: HasId> fmt::Debug for Id<T>
where
    T::IdType: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Id").field(&self.0).finish()
    }
}

impl<T: HasId> fmt::Display for Id<T>
where
    T::IdType: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: HasId> Clone for Id<T>
where
    T::IdType: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: HasId> Copy for Id<T> where T::IdType: Copy {}

impl<T: HasId> hash::Hash for Id<T>
where
    T::IdType: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: HasId> PartialEq for Id<T>
where
    T::IdType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: HasId> Eq for Id<T> where T::IdType: Eq {}

impl<'de, T: HasId> Deserialize<'de> for Id<T>
where
    T::IdType: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::IdType::deserialize(deserializer).map(|id| Id::new(id))
    }
}

impl<T: HasId> Serialize for Id<T>
where
    T::IdType: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T: HasId + JsonSchema> JsonSchema for Id<T>
where
    T::IdType: Serialize,
{
    fn schema_name() -> String {
        // Exclude the module path to make the name in generated schemas clearer.
        format!("{}Id", T::schema_name())
    }

    fn schema_id() -> Cow<'static, str> {
        // Include the module, in case a type with the same name is in another module/crate
        Cow::Borrowed(concat!(module_path!(), "::Id"))
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("id".to_owned()),
            ..Default::default()
        }
        .into()
    }
}
