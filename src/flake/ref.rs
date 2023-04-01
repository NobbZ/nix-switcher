use std::{collections::HashMap, fmt::Display, str::FromStr};

use eyre::{anyhow, Error, Result};
use url::Url;

#[derive(Debug, Clone)]
pub struct FlakeRef {
    url: Url,
}

impl FromStr for FlakeRef {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let url = Url::parse(s)?;
        Ok(Self { url })
    }
}

impl From<FlakeRef> for Url {
    #[inline]
    fn from(val: FlakeRef) -> Self {
        // TODO: use `NormalizedFlakeRef`
        val.url
    }
}

impl Display for FlakeRef {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.url.fmt(f)
    }
}

impl FlakeRef {
    #[inline]
    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }

    pub fn set_commit_id<S>(&mut self, commit_id: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let url = match self.url.scheme() {
            "github" => update_or_append_query(&mut self.url, "ref", commit_id),
            schema => return Err(anyhow!("unknown schema {}", schema)),
        };

        self.url = url;

        Ok(self.to_owned())
    }

    pub fn set_fragment<S>(&mut self, fragment: S)
    where
        S: AsRef<str>,
    {
        self.url.set_fragment(Some(fragment.as_ref()));
    }
}

#[allow(dead_code)]
pub struct NormalizedFlakreRef {
    url: Url,
}

impl FromStr for NormalizedFlakreRef {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let flake_ref: FlakeRef = FromStr::from_str(s)?;

        flake_ref.try_into()
    }
}

impl TryFrom<FlakeRef> for NormalizedFlakreRef {
    type Error = Error;

    fn try_from(_value: FlakeRef) -> Result<Self> {
        unimplemented!("NormalzedFlakeRef::try_from is not yet implemented");
    }
}

fn update_or_append_query<S1, S2>(url: &mut Url, key: S1, value: S2) -> Url
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let mut query_map: HashMap<_, _> = url.query_pairs().into_owned().collect();

    query_map
        .entry(key.as_ref().to_string())
        .and_modify(|v| *v = value.as_ref().to_string())
        .or_insert(value.as_ref().to_string());

    let mut query_vec = Vec::<(String, String)>::new();
    for (k, _) in url.query_pairs().into_owned() {
        query_vec.push((k.clone(), query_map[&k].clone()))
    }

    url.query_pairs_mut()
        .clear()
        .extend_pairs(query_vec)
        .finish();

    url.to_owned()
}
