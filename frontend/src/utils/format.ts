export function gib(v: number | null): string {
  return v == null ? "—" : `${(v / 2 ** 30).toFixed(1)} Gio`;
}

export function pct(v: number | null): string {
  return v == null ? "—" : `${v.toFixed(1)} %`;
}
