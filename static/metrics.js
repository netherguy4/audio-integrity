const AudioIntegrityMetrics = (() => {
  function createPerformanceEstimator({ windowMs = 60_000, minSampleMs = 5_000, recordIntervalMs = 1_000 } = {}) {
    let runKey = null;
    let samples = [];

    function emptyEstimate() {
      return { ready: false, validationBytesPerSecond: 0, processedBytesPerSecond: 0, etaSeconds: NaN };
    }

    return {
      update(scan, now = Date.now()) {
        const nextRunKey = scan.startedAt || scan.runId || null;
        let latest = samples[samples.length - 1];
        const countersRegressed = latest && (scan.processedBytes < latest.processedBytes || scan.readBytes < latest.readBytes);
        if (nextRunKey !== runKey || countersRegressed) {
          runKey = nextRunKey;
          samples = [];
          latest = null;
        }

        const point = {
          at: now,
          processedBytes: scan.processedBytes,
          readBytes: scan.readBytes,
        };
        if (latest && now - latest.at < recordIntervalMs) {
          samples[samples.length - 1] = point;
        } else {
          samples.push(point);
        }

        const cutoff = now - windowMs;
        while (samples.length > 2 && samples[1].at <= cutoff) samples.shift();

        const anchor = samples[0];
        const current = samples[samples.length - 1];
        const elapsedSeconds = (current.at - anchor.at) / 1_000;
        if (elapsedSeconds < minSampleMs / 1_000) return emptyEstimate();

        const validationBytesPerSecond = Math.max(0, current.readBytes - anchor.readBytes) / elapsedSeconds;
        const processedBytesPerSecond = Math.max(0, current.processedBytes - anchor.processedBytes) / elapsedSeconds;
        const remainingBytes = Math.max(0, scan.totalBytes - scan.processedBytes);
        return {
          ready: true,
          validationBytesPerSecond,
          processedBytesPerSecond,
          etaSeconds: processedBytesPerSecond > 0 ? remainingBytes / processedBytesPerSecond : NaN,
        };
      },
    };
  }

  function formatDuration(seconds, lang) {
    if (!Number.isFinite(seconds) || seconds < 0) return '—';
    if (seconds < 60) return `< 1 ${lang === 'ru' ? 'мин' : 'min'}`;
    const totalMinutes = Math.ceil(seconds / 60);
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    if (!hours) return `${minutes} ${lang === 'ru' ? 'мин' : 'min'}`;
    const hourLabel = lang === 'ru' ? 'ч' : 'h';
    const minuteLabel = lang === 'ru' ? 'мин' : 'min';
    return minutes ? `${hours} ${hourLabel} ${minutes} ${minuteLabel}` : `${hours} ${hourLabel}`;
  }

  return { createPerformanceEstimator, formatDuration };
})();

if (typeof module !== 'undefined') module.exports = AudioIntegrityMetrics;
