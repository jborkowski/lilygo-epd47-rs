use esp_hal::{
    gpio::Level,
    peripherals,
    rmt::{self, Channel, PulseCode, TxChannel, TxChannelCreator},
    time::Rate,
    Blocking,
};

pub(crate) struct Rmt<'a> {
    tx_channel: Option<Channel<Blocking, 1>>,
    rmt: peripherals::RMT<'a>,
}

impl<'a> Rmt<'a> {
    pub(crate) fn new(rmt: peripherals::RMT<'a>) -> Self {
        Rmt {
            tx_channel: None,
            rmt,
        }
    }

    fn ensure_channel(&mut self) -> Result<(), crate::Error> {
        if self.tx_channel.is_some() {
            return Ok(());
        }
        let rmt = rmt::Rmt::new(
            unsafe { self.rmt.clone_unchecked() }, // TODO: find better solution
            Rate::from_mhz(80),
        )
        .map_err(crate::Error::Rmt)?;
        let tx_channel = rmt
            .channel1
            .configure(
                unsafe { peripherals::GPIO38::steal() }, // TODO: find better solution
                rmt::TxChannelConfig::default()
                    .with_clk_divider(8)
                    .with_idle_output_level(false)
                    .with_idle_output(true)
                    .with_carrier_modulation(false)
                    .with_carrier_level(false),
            
            )
            .map_err(crate::Error::Rmt)?;
        self.tx_channel = Some(tx_channel);
        Ok(())
    }

    pub(crate) fn pulse(&mut self, high: u16, low: u16, wait: bool) -> Result<(), crate::Error> {
        self.ensure_channel()?;
        let tx_channel = self.tx_channel.take().ok_or(crate::Error::Unknown)?;
        let data = if high > 0 {
            [
                PulseCode::new(Level::High, high, Level::Low, low),
                PulseCode::empty(),
            ]
        } else {
            [
                PulseCode::new(Level::High, low, Level::Low, 0),
                PulseCode::empty(),
            ]
        };
        let tx = tx_channel.transmit(&data).map_err(crate::Error::Rmt)?;
        // FIXME: This is the culprit.. We need the channel later again but can't wait
        // due to some time sensitive operations. Not sure how to solve this
        if wait {
            self.tx_channel = Some(
                tx.wait()
                    .map_err(|(err, _)| err)
                    .map_err(crate::Error::Rmt)?,
            );
        }
        Ok(())
    }
}
