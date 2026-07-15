import { TimeSeries } from "../charts/TimeSeries";

export function MemoryUsage() {
  return (
    <TimeSeries
      series={[
        { seriesKey: "mem.used_pct", label: "Mémoire", colorToken: "--series-5" },
      ]}
      unit="%"
      range={[0, 100]}
    />
  );
}
