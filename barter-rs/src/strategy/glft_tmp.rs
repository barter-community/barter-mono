use std::*;
use std::collections::HashMap;

use numba::{njit};
use hftbacktest::{BUY, SELL};
fn measure_trading_intensity_and_volatility<T0, RT>(hbt: T0) -> RT {
let arrival_depth = np.full(10000000, np.nan, np.float64);
let mid_price_chg = np.full(10000000, np.nan, np.float64);
let mut t = 0;
let mut prev_mid_price_tick = np.nan;
let mut mid_price_tick = np.nan;
while hbt.elapse(100000) {
if !np.isnan(mid_price_tick) {
let mut depth = -(np.inf);
for trade in hbt.last_trades {
let side = trade[3];
let trade_price_tick = (trade[4]/hbt.tick_size);
if side == BUY {
depth = np.nanmax(vec![(trade_price_tick - mid_price_tick), depth]);
} else {
depth = np.nanmax(vec![(mid_price_tick - trade_price_tick), depth]);
}
}
arrival_depth[t] = depth;
}
hbt.clear_last_trades();
prev_mid_price_tick = mid_price_tick;
mid_price_tick = ((hbt.best_bid_tick + hbt.best_ask_tick)/2.0);
mid_price_chg[t] = (mid_price_tick - prev_mid_price_tick);
t += 1;
if t >= arrival_depth.len()||t >= mid_price_chg.len() {
raise!(Exception); //unsupported
}
}
return (arrival_depth[..t], mid_price_chg[..t]);
}
fn measure_trading_intensity<T0, T1, RT>(order_arrival_depth: T0, out: T1) -> RT {
let mut max_tick = 0;
for depth in order_arrival_depth {
if !np.isfinite(depth) {
continue;
}
let tick = (round((depth/0.5)) - 1);
if tick < 0||tick >= out.len() {
continue;
}
out[..tick] += 1;
max_tick = max_tick.iter().max().unwrap();
}
return out[..max_tick];
}
fn plot()  {
let tmp = np.zeros(500, np.float64);
let mut lambda_ = measure_trading_intensity(arrival_depth[..6000], tmp);
lambda_ /= 600;
let ticks = (np.arange(lambda_.len()) + 0.5);
let y = np.log(lambda_);
let (k_, logA) = linear_regression(ticks, y);
let A = np.exp(logA);
let k = -(k_);
println!("{:?} ","A={}, k={}".format(A, k));
}
fn linear_regression<T0, T1, RT>(x: T0, y: T1) -> RT {
let sx = np.sum(x);
let sy = np.sum(y);
let sx2 = np.sum(x.pow(2));
let sxy = np.sum((x*y));
let w = x.len();
let slope = (((w*sxy) - (sx*sy))/((w*sx2) - sx.pow(2)));
let intercept = ((sy - (slope*sx))/w);
return (slope, intercept);
}
fn compute_coeff<T0, T1, T2, T3, T4, RT>(xi: T0, gamma: T1, delta: T2, A: T3, k: T4) -> RT {
let inv_k = np.divide(1, k);
let c1 = ((1/(xi*delta))*np.log((1 + ((xi*delta)*inv_k))));
let c2 = np.sqrt((np.divide(gamma, (((2*A)*delta)*k))*(1 + ((xi*delta)*inv_k)).pow(((k/(xi*delta)) + 1))));
return (c1, c2);
}
fn rest()  {
if (t % 50) == 0 {
if t >= (6000 - 1) {
tmp[..] = 0;
let mut lambda_ = measure_trading_intensity(arrival_depth[((t + 1) - 6000)..(t + 1)], tmp);
lambda_ = (lambda_[..70]/600);
let x = ticks[..lambda_.len()];
let y = np.log(lambda_);
let (k_, logA) = linear_regression(x, y);
let A = np.exp(logA);
let k = -(k_);
let volatility = (np.nanstd(mid_price_chg[((t + 1) - 6000)..(t + 1)])*np.sqrt(10));
}
}
let (c1, c2) = compute_coeff(gamma, gamma, delta, A, k);
let half_spread = (c1 + (((delta/2)*c2)*volatility));
let skew = (c2*volatility);
let bid_depth = (half_spread + (skew*hbt.position));
let ask_depth = (half_spread - (skew*hbt.position));
let bid_price = (np.round((mid_price_tick - bid_depth)).iter().min().unwrap()*hbt.tick_size);
let ask_price = (np.round((mid_price_tick + ask_depth)).iter().max().unwrap()*hbt.tick_size);
hbt.clear_inactive_orders();
for order in hbt.orders.values() {
if order.side == BUY&&order.cancellable&&order.price != bid_price {
hbt.cancel(order.order_id);
}
if order.side == SELL&&order.cancellable&&order.price != ask_price {
hbt.cancel(order.order_id);
}
}
if hbt.position < max_position&&np.isfinite(bid_price) {
let bid_price_as_order_id = round((bid_price/hbt.tick_size));
if hbt.orders.iter().all(|&x| x != bid_price_as_order_id) {
hbt.submit_buy_order(bid_price_as_order_id, bid_price, order_qty, GTX);
}
}
if hbt.position > -(max_position)&&np.isfinite(ask_price) {
let ask_price_as_order_id = round((ask_price/hbt.tick_size));
if hbt.orders.iter().all(|&x| x != ask_price_as_order_id) {
hbt.submit_sell_order(ask_price_as_order_id, ask_price, order_qty, GTX);
}
}
out[(t, 0)] = half_spread;
out[(t, 1)] = skew;
out[(t, 2)] = volatility;
out[(t, 3)] = A;
out[(t, 4)] = k;
t += 1;
if t >= arrival_depth.len()||t >= mid_price_chg.len()||t >= out.len() {
raise!(Exception); //unsupported
}
stat.record(hbt);
}