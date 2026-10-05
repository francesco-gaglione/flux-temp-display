#[derive(Debug)]
pub struct Temperature {
    temperature: u16,
}

impl Temperature {
    pub fn new(temperature: u16) -> Self {
        Self { temperature }
    }

    pub fn temperature(&self) -> u16 {
        self.temperature
    }
}
