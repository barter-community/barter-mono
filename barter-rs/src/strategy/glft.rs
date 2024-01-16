use std::*;

use barter_data::subscription::trade::PublicTrade;
use barter_integration::model::Side;
use ndarray::prelude::*;
use ndarray_stats::{QuantileExt, SummaryStatisticsExt};
use plotpy::{Curve, Plot, StrError};

#[derive(Clone, Debug, Copy)]
pub struct MeasurementParams {
    pub best_bid_tick: f64,
    pub best_ask_tick: f64,
    pub tick_size: f64,
    pub mid_price_tick: f64,
    pub index: usize,
}

impl Default for MeasurementParams {
    fn default() -> Self {
        Self {
            best_bid_tick: f64::NAN,
            best_ask_tick: f64::NAN,
            mid_price_tick: f64::NAN,
            tick_size: 0.01,
            index: 0,
        }
    }
}

pub fn measure_trading_intensity_and_volatility<'a>(
    last_trades: &'a mut Vec<PublicTrade>,
    m: &mut MeasurementParams,
    mut arrival_depth: ArrayViewMut<'a, f64, Ix1>,
    mut mid_price_chg: ArrayViewMut<'a, f64, Ix1>,
) {
    if m.mid_price_tick != f64::NAN {
        let mut depth = -(f64::INFINITY);
        for trade in last_trades {
            let side = trade.side;
            let trade_price_tick = trade.price / m.tick_size;
            if side == Side::Buy {
                depth = array![trade_price_tick - m.mid_price_tick, depth]
                    .max_skipnan()
                    .clone();
            } else {
                depth = array![m.mid_price_tick - trade_price_tick, depth]
                    .max_skipnan()
                    .clone();
            }
        }
        arrival_depth[m.index] = depth;
    }

    let prev_mid_price_tick = m.mid_price_tick;
    m.mid_price_tick = (m.best_bid_tick + m.best_ask_tick) / 2.0;

    mid_price_chg[m.index] = m.mid_price_tick - prev_mid_price_tick;

    m.index += 1;
    if m.index >= arrival_depth.len() || m.index >= mid_price_chg.len() {
        panic!("t >= arrival_depth.len() || t >= mid_price_chg.len()");
    }
}

pub fn measure_trading_intensity(
    arrival_depth: ArrayView<f64, Ix1>,
    mut out: Array<usize, Ix1>,
) -> Array<usize, Ix1> {
    let mut max_tick = 0;
    for depth in arrival_depth {
        if depth.is_infinite() {
            continue;
        }
        let tick = (depth / 0.5).round() as isize - 1;
        if tick < 0 || tick >= out.len() as isize {
            continue;
        }
        out.slice_mut(s![..tick]).mapv_inplace(|x| x + 1);
        max_tick = cmp::max(max_tick, tick);
    }
    out.slice_mut(s![..max_tick]).to_owned()
}

#[allow(non_snake_case)]
pub fn get_params(
    arrival_depth: ArrayView<'_, f64, Ix1>,
    mid_price_chg: ArrayView<'_, f64, Ix1>,
    index: usize,
) -> Result<(f64, f64, f64), StrError> {
    let tmp = Array::from_elem(500, 0);
    let a_depth = arrival_depth.slice(s![index + 1 - 6000..index]);
    let mut lambda_ = measure_trading_intensity(a_depth, tmp).map(|e| *e as f64);
    lambda_ /= 600.0;
    // trim data down to get better fit
    lambda_ = lambda_.slice_move(s![..120]);
    let ticks = Array::range(0.0, lambda_.len() as f64, 1.0) + 0.5;

    let y = lambda_.map(|e| e.ln());
    let (k_, log_a) = linear_regression(&ticks, &y);
    let A = log_a.exp();
    let k = -(k_);

    // configure curve
    // let mut curve = Curve::new();
    // curve.set_line_width(2.0);

    // curve.draw(&ticks.to_vec(), &lambda_.to_vec());

    // let fitted = A * (-k * &ticks).map(|e| e.exp());
    // curve.draw(&ticks.to_vec(), &fitted.to_vec());

    // add curve to plot
    // let mut plot = Plot::new();
    // plot.add(&curve)
    //     .grid_and_labels("delta (ticks from the mid-price)", "Count (per second)");

    // // save figure
    // plot.save("./plot/doc_curve_methods.svg").unwrap();

    // Volatility
    // let firstNan = mid_price_chg.iter().position(|&x| x.is_nan()).unwrap();
    // let mid_price_chg = mid_price_chg.slice_move(s![firstNan + 1..]);
    // let secondNan = mid_price_chg.iter().position(|&x| x.is_nan()).unwrap();
    // let mid_price_chg = mid_price_chg.slice_move(s![..secondNan]);

    let mid_price_chg = mid_price_chg.slice(s![index + 1 - 6000..index]);
    let weights = Array::from_elem(mid_price_chg.len(), 1_f64);

    // mid_price_chg
    //     .iter()
    //     .for_each(|p| println!("mid price: {:?}", p));

    // Since we need volatility in ticks per square root of a second and our measurement is every 100ms,
    // multiply by the square root of 10.
    let volatility = mid_price_chg
        .view()
        .weighted_std(&weights.view(), 0.0)
        .unwrap()
        * 10.0_f64.sqrt();

    Ok((A, k, volatility))
}

fn linear_regression(x: &Array<f64, Ix1>, y: &Array<f64, Ix1>) -> (f64, f64) {
    let sx = x.sum();
    let sy = y.sum();
    let sx2 = x.mapv(|a| a.powi(2)).sum();
    let w = x.len() as f64;
    let sxy = (x * y).sum();
    let slope = ((w * sxy) - (sx * sy)) / ((w * sx2) - sx.powi(2));
    let intercept = (sy - (slope * sx)) / w;
    return (slope, intercept);
}

#[allow(non_snake_case)]
pub fn compute_coeff(xi: f64, gamma: f64, delta: f64, A: f64, k: f64) -> (f64, f64) {
    let inv_k = 1.0 / k;
    let c1 = (1.0 / (xi * delta)) * f64::ln(1.0 + ((xi * delta) * inv_k));
    let c2 = f64::sqrt(
        (gamma / (((2.0 * A) * delta) * k))
            * (1.0 + ((xi * delta) * inv_k)).powf(k / (xi * delta) + 1.0),
    );
    return (c1, c2);
}
