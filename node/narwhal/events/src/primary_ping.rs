// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimaryPing<N: Network> {
    pub version: u32,
    pub block_locators: BlockLocators<N>,
    pub primary_certificate: Data<BatchCertificate<N>>,
    pub batch_certificates: Vec<Data<BatchCertificate<N>>>,
}

impl<N: Network> PrimaryPing<N> {
    /// Initializes a new ping event.
    pub const fn new(
        version: u32,
        block_locators: BlockLocators<N>,
        primary_certificate: Data<BatchCertificate<N>>,
        batch_certificates: Vec<Data<BatchCertificate<N>>>,
    ) -> Self {
        Self { version, block_locators, primary_certificate, batch_certificates }
    }
}

impl<N: Network> From<(u32, BlockLocators<N>, BatchCertificate<N>, Vec<BatchCertificate<N>>)> for PrimaryPing<N> {
    /// Initializes a new ping event.
    fn from(
        (version, block_locators, primary_certificate, batch_certificates): (
            u32,
            BlockLocators<N>,
            BatchCertificate<N>,
            Vec<BatchCertificate<N>>,
        ),
    ) -> Self {
        Self::new(
            version,
            block_locators,
            Data::Object(primary_certificate),
            batch_certificates.into_iter().map(Data::Object).collect(),
        )
    }
}

impl<N: Network> EventTrait for PrimaryPing<N> {
    /// Returns the event name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "PrimaryPing".into()
    }
}

impl<N: Network> ToBytes for PrimaryPing<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.version.write_le(&mut writer)?;
        self.block_locators.write_le(&mut writer)?;
        self.primary_certificate.write_le(&mut writer)?;
        u16::try_from(self.batch_certificates.len()).map_err(error)?.write_le(&mut writer)?;
        for batch_certificate in &self.batch_certificates {
            batch_certificate.write_le(&mut writer)?;
        }
        Ok(())
    }
}

impl<N: Network> FromBytes for PrimaryPing<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let version = u32::read_le(&mut reader)?;
        let block_locators = BlockLocators::read_le(&mut reader)?;
        let primary_certificate = Data::read_le(&mut reader)?;
        let num_certificates = u16::read_le(&mut reader)?;
        // TODO: Limit to the number of MAX_COMMITTEE + 1 (self).
        let mut batch_certificates = Vec::with_capacity(usize::from(num_certificates));
        for _ in 0..num_certificates {
            batch_certificates.push(Data::read_le(&mut reader)?);
        }
        Ok(Self::new(version, block_locators, primary_certificate, batch_certificates))
    }
}

#[cfg(test)]
pub mod prop_tests {
    use crate::{certificate_response::prop_tests::any_batch_certificate, PrimaryPing};
    use snarkos_node_sync_locators::{test_helpers::sample_block_locators, BlockLocators};
    use snarkvm::utilities::{FromBytes, ToBytes};

    use bytes::{Buf, BufMut, BytesMut};
    use proptest::prelude::{any, BoxedStrategy, Strategy};
    use test_strategy::proptest;

    type CurrentNetwork = snarkvm::prelude::Testnet3;

    pub fn any_block_locators() -> BoxedStrategy<BlockLocators<CurrentNetwork>> {
        any::<u32>().prop_map(sample_block_locators).boxed()
    }

    pub fn any_primary_ping() -> BoxedStrategy<PrimaryPing<CurrentNetwork>> {
        (any::<u32>(), any_block_locators(), any_batch_certificate())
            .prop_map(|(version, block_locators, batch_certificate)| {
                PrimaryPing::from((version, block_locators, batch_certificate.clone(), vec![batch_certificate]))
            })
            .boxed()
    }

    #[proptest]
    fn primary_ping_roundtrip(#[strategy(any_primary_ping())] primary_ping: PrimaryPing<CurrentNetwork>) {
        let mut bytes = BytesMut::default().writer();
        primary_ping.write_le(&mut bytes).unwrap();
        let decoded = PrimaryPing::<CurrentNetwork>::read_le(&mut bytes.into_inner().reader()).unwrap();
        assert_eq!(primary_ping, decoded);
    }
}
