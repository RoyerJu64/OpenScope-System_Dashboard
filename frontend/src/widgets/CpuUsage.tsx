import { TimeSeries } from "../charts/TimeSeries";

export function CpuUsage() {
  return (
    <TimeSeries
      series={[{ seriesKey: "cpu.usage", label: "CPU", colorToken: "--series-1" }]}
      unit="%"
      range={[0, 100]}
    />
  );
}
