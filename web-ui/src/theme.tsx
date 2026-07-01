// Light-only per DESIGN.md. Kept as a no-op shim so any lingering imports
// resolve; there is no theme switching in the product.
export function useTheme() {
  return { theme: "light" as const };
}
