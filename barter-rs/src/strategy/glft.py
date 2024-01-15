from numba import njit
from hftbacktest import BUY, SELL

import numpy as np

@njit
def measure_trading_intensity_and_volatility(hbt):
    arrival_depth = np.full(10_000_000, np.nan, np.float64)
    mid_price_chg = np.full(10_000_000, np.nan, np.float64)

    t = 0
    prev_mid_price_tick = np.nan
    mid_price_tick = np.nan
    
    # Checks every 100 milliseconds.
    while hbt.elapse(100_000):
        #--------------------------------------------------------
        # Records market order's arrival depth from the mid-price.
        if not np.isnan(mid_price_tick):
            depth = -np.inf
            for trade in hbt.last_trades:
                side = trade[3]
                trade_price_tick = trade[4] / hbt.tick_size
                
                if side == BUY:
                    depth = np.nanmax([trade_price_tick - mid_price_tick, depth])
                else:
                    depth = np.nanmax([mid_price_tick - trade_price_tick, depth])
            arrival_depth[t] = depth
        
        hbt.clear_last_trades()
        
        prev_mid_price_tick = mid_price_tick
        mid_price_tick = (hbt.best_bid_tick + hbt.best_ask_tick) / 2.0
        
        # Records the mid-price change for volatility calculation.
        mid_price_chg[t] = mid_price_tick - prev_mid_price_tick
        
        t += 1
        if t >= len(arrival_depth) or t >= len(mid_price_chg):
            raise Exception
    return arrival_depth[:t], mid_price_chg[:t]
  
@njit
def measure_trading_intensity(order_arrival_depth, out):
    max_tick = 0
    for depth in order_arrival_depth:
        if not np.isfinite(depth):
            continue
        
        # Sets the tick index to 0 for the nearest possible best price 
        # as the order arrival depth in ticks is measured from the mid-price
        tick = round(depth / .5) - 1
        
        # In a fast-moving market, buy trades can occur below the mid-price (and vice versa for sell trades) 
        # since the mid-price is measured in a previous time-step; 
        # however, to simplify the problem, we will exclude those cases.
        if tick < 0 or tick >= len(out):
            continue
        
        # All of our possible quotes within the order arrival depth, 
        # excluding those at the same price, are considered executed.
        out[:tick] += 1
        
        max_tick = max(max_tick, tick)
    return out[:max_tick]
  
def plot():
    tmp = np.zeros(500, np.float64) 

    # Measures trading intensity (lambda) for the first 10-minute window.
    lambda_ = measure_trading_intensity(arrival_depth[:6_000], tmp)

    # Since it is measured for a 10-minute window, divide by 600 to convert it to per second.
    lambda_ /= 600

    # Creates ticks from the mid-price.
    ticks = np.arange(len(lambda_)) + .5
    
    y = np.log(lambda_)
    k_, logA = linear_regression(ticks, y)
    A = np.exp(logA)
    k = -k_

    print('A={}, k={}'.format(A, k))
    
@njit
def linear_regression(x, y):
    sx = np.sum(x)
    sy = np.sum(y)
    sx2 = np.sum(x ** 2)
    sxy = np.sum(x * y)
    w = len(x)
    slope = (w * sxy - sx * sy) / (w * sx2 - sx**2)
    intercept = (sy - slope * sx) / w
    return slope, intercept
  
@njit
def compute_coeff(xi, gamma, delta, A, k):
    inv_k = np.divide(1, k)
    c1 = 1 / (xi * delta) * np.log(1 + xi * delta * inv_k)
    c2 = np.sqrt(np.divide(gamma, 2 * A * delta * k) * ((1 + xi * delta * inv_k) ** (k / (xi * delta) + 1)))
    return c1, c2
  
@njit
def rest(): 
    #--------------------------------------------------------
    # Calibrates A, k and calculates the market volatility.
    
    # Updates A, k, and the volatility every 5-sec.
    if t % 50 == 0:
        # Window size is 10-minute.
        if t >= 6_000 - 1:
            # Calibrates A, k
            tmp[:] = 0
            lambda_ = measure_trading_intensity(arrival_depth[t + 1 - 6_000:t + 1], tmp)
            lambda_ = lambda_[:70] / 600
            x = ticks[:len(lambda_)]
            y = np.log(lambda_)
            k_, logA = linear_regression(x, y)
            A = np.exp(logA)
            k = -k_
        
            # Updates the volatility.
            volatility = np.nanstd(mid_price_chg[t + 1 - 6_000:t + 1]) * np.sqrt(10)

    #--------------------------------------------------------
    # Computes bid price and ask price.

    c1, c2 = compute_coeff(gamma, gamma, delta, A, k)
    
    half_spread = c1 + delta / 2 * c2 * volatility
    skew = c2 * volatility
    
    bid_depth = half_spread + skew * hbt.position
    ask_depth = half_spread - skew * hbt.position

    bid_price = min(np.round(mid_price_tick - bid_depth), hbt.best_bid_tick) * hbt.tick_size
    ask_price = max(np.round(mid_price_tick + ask_depth), hbt.best_ask_tick) * hbt.tick_size
    
    #--------------------------------------------------------
    # Updates quotes.
    
    hbt.clear_inactive_orders()
    
    # Cancel orders if they differ from the updated bid and ask prices.
    for order in hbt.orders.values():
        if order.side == BUY and order.cancellable and order.price != bid_price:
            hbt.cancel(order.order_id)
        if order.side == SELL and order.cancellable and order.price != ask_price:
            hbt.cancel(order.order_id)

    # If the current position is within the maximum position,
    # submit the new order only if no order exists at the same price.
    if hbt.position < max_position and np.isfinite(bid_price):
        bid_price_as_order_id = round(bid_price / hbt.tick_size)
        if bid_price_as_order_id not in hbt.orders:
            hbt.submit_buy_order(bid_price_as_order_id, bid_price, order_qty, GTX)
    if hbt.position > -max_position and np.isfinite(ask_price):
        ask_price_as_order_id = round(ask_price / hbt.tick_size)
        if ask_price_as_order_id not in hbt.orders:
            hbt.submit_sell_order(ask_price_as_order_id, ask_price, order_qty, GTX)
            
    #--------------------------------------------------------
    # Records variables and stats for analysis.
    
    out[t, 0] = half_spread
    out[t, 1] = skew
    out[t, 2] = volatility
    out[t, 3] = A
    out[t, 4] = k
    
    t += 1
    
    if t >= len(arrival_depth) or t >= len(mid_price_chg) or t >= len(out):
        raise Exception
    
    # Records the current state for stat calculation.
    stat.record(hbt)