import { TimeSeries } from "../charts/TimeSeries";

export function NetworkIo() {
  return (
    <TimeSeries
      series={[
        { seriesKey: "net.rx_bps", label: "Réception" },
        { seriesKey: "net.tx_bps", label: "Émission" },
      ]}
      format="bytes"
    />
  );
}
