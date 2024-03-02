use crate::model::Price;

#[derive(Default)]
pub struct MovingAverage {
    value: Price,
    length: usize,
}

impl MovingAverage {
    pub fn feed(&mut self, value: Price, times: usize) {
        self.value += value * times as Price;
        self.length += times;
    }

    pub fn clear(&mut self) {
        self.value = Price::default();
        self.length = 0;
    }

    pub fn avg(&self) -> f64 {
        self.value / self.length as Price
    }
}
