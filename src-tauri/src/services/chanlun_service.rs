use crate::models::market_data::KlineData;
use crate::models::signal_alert::{
    Bi, ChanlunAnalysis, ChanlunSignal, Fractal, Pivot,
};

pub struct ChanlunService;

impl ChanlunService {
    pub fn analyze(klines: &[KlineData], symbol: &str) -> ChanlunAnalysis {
        if klines.is_empty() {
            return ChanlunAnalysis {
                symbol: symbol.to_string(),
                fractals: Vec::new(),
                bis: Vec::new(),
                pivots: Vec::new(),
                signals: Vec::new(),
                current_trend: "unknown".to_string(),
            };
        }

        let fractals = Self::detect_fractals(klines);
        let bis = Self::detect_bis(&fractals);
        let pivots = Self::detect_pivots(&bis);
        let signals = Self::detect_signals(klines, &fractals, &bis, &pivots);

        let current_trend = if bis.is_empty() {
            "unknown".to_string()
        } else {
            bis.last().unwrap().direction.clone()
        };

        ChanlunAnalysis {
            symbol: symbol.to_string(),
            fractals,
            bis,
            pivots,
            signals,
            current_trend,
        }
    }

    fn detect_fractals(klines: &[KlineData]) -> Vec<Fractal> {
        let mut fractals = Vec::new();
        if klines.len() < 3 {
            return fractals;
        }

        for i in 1..klines.len() - 1 {
            let prev_high = klines[i - 1].high;
            let curr_high = klines[i].high;
            let next_high = klines[i + 1].high;
            let prev_low = klines[i - 1].low;
            let curr_low = klines[i].low;
            let next_low = klines[i + 1].low;

            // Top fractal: curr high > neighbors high
            if curr_high > prev_high && curr_high > next_high {
                fractals.push(Fractal {
                    index: i as u32,
                    fractal_type: "top".to_string(),
                    value: curr_high,
                });
            }

            // Bottom fractal: curr low < neighbors low
            if curr_low < prev_low && curr_low < next_low {
                fractals.push(Fractal {
                    index: i as u32,
                    fractal_type: "bottom".to_string(),
                    value: curr_low,
                });
            }
        }

        // Deduplicate: same index can't be both top and bottom, keep first
        let mut seen = std::collections::HashSet::new();
        fractals.retain(|f| seen.insert(f.index));

        fractals.sort_by_key(|f| f.index);
        fractals
    }

    fn detect_bis(fractals: &[Fractal]) -> Vec<Bi> {
        let mut bis = Vec::new();
        if fractals.len() < 2 {
            return bis;
        }

        // Merge adjacent same-type fractals, keeping the extreme one
        let merged = Self::merge_fractals(fractals);

        // Connect alternating top-bottom fractals
        for i in 0..merged.len().saturating_sub(1) {
            let curr = &merged[i];
            let next = &merged[i + 1];

            if curr.fractal_type != next.fractal_type {
                let direction = if next.value > curr.value {
                    "up".to_string()
                } else {
                    "down".to_string()
                };

                bis.push(Bi {
                    start_index: curr.index,
                    end_index: next.index,
                    direction,
                    start_value: curr.value,
                    end_value: next.value,
                });
            }
        }

        bis
    }

    fn merge_fractals(fractals: &[Fractal]) -> Vec<Fractal> {
        if fractals.is_empty() {
            return Vec::new();
        }

        let mut merged = Vec::new();
        let mut current = fractals[0].clone();

        for f in &fractals[1..] {
            if f.fractal_type == current.fractal_type {
                if f.fractal_type == "top" && f.value > current.value {
                    current = f.clone();
                } else if f.fractal_type == "bottom" && f.value < current.value {
                    current = f.clone();
                }
            } else {
                merged.push(current);
                current = f.clone();
            }
        }
        merged.push(current);

        merged
    }

    fn detect_pivots(bis: &[Bi]) -> Vec<Pivot> {
        let mut pivots = Vec::new();
        if bis.len() < 3 {
            return pivots;
        }

        // A pivot requires at least 3 overlapping bis
        for i in 0..bis.len().saturating_sub(2) {
            let b1 = &bis[i];
            let b2 = &bis[i + 1];
            let b3 = &bis[i + 2];

            let highs = [
                b1.start_value.max(b1.end_value),
                b2.start_value.max(b2.end_value),
                b3.start_value.max(b3.end_value),
            ];
            let lows = [
                b1.start_value.min(b1.end_value),
                b2.start_value.min(b2.end_value),
                b3.start_value.min(b3.end_value),
            ];

            let zg = highs.iter().cloned().fold(f64::MAX, f64::min);
            let zd = lows.iter().cloned().fold(f64::MIN, f64::max);

            if zg > zd {
                pivots.push(Pivot {
                    start_index: b1.start_index,
                    end_index: b3.end_index,
                    zg,
                    zd,
                    zz: (zg + zd) / 2.0,
                });
            }
        }

        pivots
    }

    fn detect_signals(
        klines: &[KlineData],
        _fractals: &[Fractal],
        bis: &[Bi],
        pivots: &[Pivot],
    ) -> Vec<ChanlunSignal> {
        let mut signals = Vec::new();
        if klines.is_empty() || pivots.is_empty() || bis.is_empty() {
            return signals;
        }

        let last_price = klines.last().unwrap().close;
        let last_bi = bis.last().unwrap();
        let last_pivot = pivots.last().unwrap();

        // Buy signal: price breaks above pivot ZG (pivot upper boundary) from below
        if last_price > last_pivot.zg && last_bi.direction == "up" {
            signals.push(ChanlunSignal {
                signal_type: "buy3".to_string(),
                index: (klines.len() - 1) as u32,
                price: last_price,
                description: format!("三买: 价格 {} 突破中枢上沿 {}", last_price, last_pivot.zg),
            });
        }

        // Sell signal: price breaks below pivot ZD (pivot lower boundary) from above
        if last_price < last_pivot.zd && last_bi.direction == "down" {
            signals.push(ChanlunSignal {
                signal_type: "sell3".to_string(),
                index: (klines.len() - 1) as u32,
                price: last_price,
                description: format!("三卖: 价格 {} 跌破中枢下沿 {}", last_price, last_pivot.zd),
            });
        }

        // Buy signal: price returns to pivot center and bounces (second buy point)
        if last_price > last_pivot.zz && last_price < last_pivot.zg
            && last_bi.direction == "up"
            && bis.len() >= 2
        {
            let prev_bi = &bis[bis.len() - 2];
            if prev_bi.direction == "down" && prev_bi.end_value < last_pivot.zd {
                signals.push(ChanlunSignal {
                    signal_type: "buy2".to_string(),
                    index: (klines.len() - 1) as u32,
                    price: last_price,
                    description: format!("二买: 价格 {} 回踩中枢后企稳", last_price),
                });
            }
        }

        signals
    }
}
