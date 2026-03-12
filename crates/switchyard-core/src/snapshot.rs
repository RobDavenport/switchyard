use crate::ids::SignalId;
use crate::runtime::TaskRecord;
#[cfg(feature = "serde")]
use alloc::vec::Vec;
#[cfg(feature = "serde")]
use core::fmt;
#[cfg(feature = "serde")]
use serde::ser::SerializeSeq;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSnapshot<const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize> {
    pub clock: u64,
    pub next_task_id: u32,
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "serialize_array", deserialize_with = "deserialize_array")
    )]
    pub tasks: [Option<TaskRecord>; MAX_TASKS],
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "serialize_array", deserialize_with = "deserialize_array")
    )]
    pub pending_signals: [Option<SignalId>; MAX_PENDING_SIGNALS],
}

impl<const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize>
    RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>
{
    pub fn empty() -> Self {
        Self {
            clock: 0,
            next_task_id: 0,
            tasks: core::array::from_fn(|_| None),
            pending_signals: core::array::from_fn(|_| None),
        }
    }
}

impl<const MAX_TASKS: usize, const MAX_PENDING_SIGNALS: usize> Default
    for RuntimeSnapshot<MAX_TASKS, MAX_PENDING_SIGNALS>
{
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(feature = "serde")]
fn serialize_array<S, T, const N: usize>(values: &[T; N], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let mut sequence = serializer.serialize_seq(Some(N))?;
    for value in values {
        sequence.serialize_element(value)?;
    }
    sequence.end()
}

#[cfg(feature = "serde")]
fn deserialize_array<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let values = Vec::<T>::deserialize(deserializer)?;
    values.try_into().map_err(|values: Vec<T>| {
        serde::de::Error::invalid_length(values.len(), &ExpectedArrayLength::<N>)
    })
}

#[cfg(feature = "serde")]
struct ExpectedArrayLength<const N: usize>;

#[cfg(feature = "serde")]
impl<const N: usize> serde::de::Expected for ExpectedArrayLength<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "an array with {} elements", N)
    }
}
