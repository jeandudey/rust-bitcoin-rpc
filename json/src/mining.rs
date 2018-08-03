// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Mining related RPC result types.

use std::fmt::{self, Formatter};
use std::str::FromStr;

use serde::{de, ser};
use strason::Json;

/// Models the result of "estimatesmartfee"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EstimateSmartFee {
    /// Estimate fee rate in BTC/kB.
    pub feerate: Option<Json>,
    /// Errors encountered during processing.
    pub errors: Option<Vec<String>>,
    /// Block number where estimate was found.
    pub blocks: i64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum EstimateMode {
    Unset,
    Economical,
    Conservative,
}

impl FromStr for EstimateMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNSET" => Ok(EstimateMode::Unset),
            "ECONOMICAL" => Ok(EstimateMode::Economical),
            "CONSERVATIVE" => Ok(EstimateMode::Conservative),
            _ => Err(()),
        }
    }
}

impl<'de> de::Deserialize<'de> for EstimateMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = EstimateMode;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "estimate mode")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                EstimateMode::from_str(v)
                    .map_err(|_e| de::Error::custom("invalid string"))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                EstimateMode::from_str(v)
                    .map_err(|_e| de::Error::custom("invalid string"))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                EstimateMode::from_str(&*v)
                    .map_err(|_e| de::Error::custom("invalid string"))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl ser::Serialize for EstimateMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let s = match *self {
            EstimateMode::Unset => "UNSET",
            EstimateMode::Economical => "ECONOMICAL",
            EstimateMode::Conservative => "CONSERVATIVE",
        };

        serializer.serialize_str(s)
    }
}

