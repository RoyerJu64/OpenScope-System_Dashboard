import { TimeSeries } from "../charts/TimeSeries";

export function DiskIo() {
  return (
    <TimeSeries
      series={[
        { seriesKey: "disk.read_bps", label: "Lecture" },
        { seriesKey: "disk.write_bps", label: "Écriture" },
      ]}
      format="bytes"
    />
  );
}
