const test = require('node:test');
const assert = require('node:assert/strict');
const { createPerformanceEstimator, formatDuration } = require('./metrics.js');

test('uses recent counter deltas instead of lifetime averages', () => {
  const estimator = createPerformanceEstimator({ minSampleMs: 5_000, recordIntervalMs: 0 });
  const scan = { startedAt: 'run-1', totalBytes: 1_000, processedBytes: 300, readBytes: 100 };

  assert.equal(estimator.update(scan, 0).ready, false);
  const estimate = estimator.update({ ...scan, processedBytes: 500, readBytes: 200 }, 10_000);

  assert.equal(estimate.validationBytesPerSecond, 10);
  assert.equal(estimate.processedBytesPerSecond, 20);
  assert.equal(estimate.etaSeconds, 25);
});

test('counts cached progress for ETA without calling it validation throughput', () => {
  const estimator = createPerformanceEstimator({ minSampleMs: 5_000, recordIntervalMs: 0 });
  const scan = { startedAt: 'run-1', totalBytes: 1_000, processedBytes: 100, readBytes: 50 };

  estimator.update(scan, 0);
  const estimate = estimator.update({ ...scan, processedBytes: 600 }, 10_000);

  assert.equal(estimate.validationBytesPerSecond, 0);
  assert.equal(estimate.processedBytesPerSecond, 50);
  assert.equal(estimate.etaSeconds, 8);
});

test('resets samples when a new scan starts', () => {
  const estimator = createPerformanceEstimator({ minSampleMs: 5_000, recordIntervalMs: 0 });
  estimator.update({ startedAt: 'run-1', totalBytes: 100, processedBytes: 10, readBytes: 10 }, 0);
  estimator.update({ startedAt: 'run-1', totalBytes: 100, processedBytes: 20, readBytes: 20 }, 10_000);

  const estimate = estimator.update({ startedAt: 'run-2', totalBytes: 100, processedBytes: 0, readBytes: 0 }, 20_000);

  assert.equal(estimate.ready, false);
});

test('normalizes rounded minutes into hours', () => {
  assert.equal(formatDuration(59, 'ru'), '< 1 мин');
  assert.equal(formatDuration(3_599, 'ru'), '1 ч');
  assert.equal(formatDuration(3_601, 'en'), '1 h 1 min');
  assert.equal(formatDuration(NaN, 'en'), '—');
});
