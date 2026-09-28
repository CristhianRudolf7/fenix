---
name: time-series-forecasting
description: Forecast time-dependent data with leakage-free backtesting and explicit uncertainty.
metadata:
  version: 1.0.0
  category: data
  tags: time, series, forecasting
  aliases: pronóstico, serie temporal, forecasting, ventas futuras
  required_tools: file_read,file_write,command_run
  requires: Time-indexed data and available forecasting libraries
  effects: local-write
---
# Time Series Forecasting

## Scope

Forecast time-dependent data with leakage-free backtesting and explicit uncertainty.

## Inputs

Frequency, horizon, target definition, history and known future covariates. Prerequisite: Time-indexed data and available forecasting libraries.

## Workflow

1. Inspect missing periods, duplicates, seasonality and structural changes; distinguish zero activity from absent observations.
2. Establish a naive/seasonal baseline and time-ordered validation. Fit transformations only on each training window.
3. Use only covariates available at forecast time. Compare rolling-origin errors across horizons and relevant segments.
4. Deliver forecasts with uncertainty and assumptions; explain where future conditions differ from the training period.

## Verification

Check timestamp alignment, horizon, units and baseline performance. Preserve all backtest windows, not only favorable ones.

## Limits

A foundation model is optional, not a reason to skip baselines or resource checks. Forecasts are not commitments.
