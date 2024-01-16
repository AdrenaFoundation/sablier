use crate::{ClockData, ExecContext, SerializableInstruction, Thread, Trigger};
use anchor_lang::{prelude::*, AccountDeserialize};

#[derive(Clone, Debug, PartialEq)]
pub enum VersionedThread {
    V1(Thread),
}

impl VersionedThread {
    pub fn authority(&self) -> Pubkey {
        match self {
            Self::V1(t) => t.authority,
        }
    }

    pub fn created_at(&self) -> ClockData {
        match self {
            Self::V1(t) => t.created_at.clone(),
        }
    }

    pub fn exec_context(&self) -> Option<ExecContext> {
        match self {
            Self::V1(t) => t.exec_context,
        }
    }

    pub fn id(&self) -> Vec<u8> {
        match self {
            Self::V1(t) => t.id.clone(),
        }
    }

    pub fn next_instruction(&self) -> Option<SerializableInstruction> {
        match self {
            Self::V1(t) => t.next_instruction.clone(),
        }
    }

    pub fn paused(&self) -> bool {
        match self {
            Self::V1(t) => t.paused,
        }
    }

    pub fn program_id(&self) -> Pubkey {
        match self {
            Self::V1(_) => crate::ID,
        }
    }

    pub fn pubkey(&self) -> Pubkey {
        match self {
            Self::V1(_) => Thread::pubkey(self.authority(), self.id()),
        }
    }

    pub fn rate_limit(&self) -> u64 {
        match self {
            Self::V1(t) => t.rate_limit,
        }
    }

    pub fn trigger(&self) -> Trigger {
        match self {
            Self::V1(t) => t.trigger.clone(),
        }
    }
}

impl AccountDeserialize for VersionedThread {
    fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        Self::try_deserialize_unchecked(buf)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        // Try first to deserialize into ThreadV2.
        // If this fails, try to deserialize into ThreadV1.
        Ok(VersionedThread::V1(Thread::try_deserialize(buf)?))
    }
}

impl TryFrom<Vec<u8>> for VersionedThread {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        VersionedThread::try_deserialize(&mut data.as_slice())
    }
}
