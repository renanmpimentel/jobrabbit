/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  // Light-only per DESIGN.md — no dark mode.
  theme: {
    extend: {
      colors: {
        // Surfaces
        bg: "rgb(var(--bg) / <alpha-value>)",
        surface: "rgb(var(--surface) / <alpha-value>)",
        "surface-2": "rgb(var(--surface-2) / <alpha-value>)",
        sidebar: "rgb(var(--sidebar) / <alpha-value>)",
        // DESIGN.md-named aliases for surfaces
        "bg-base": "rgb(var(--surface) / <alpha-value>)",
        "bg-subtle": "rgb(var(--bg) / <alpha-value>)",
        "bg-muted": "rgb(var(--surface-2) / <alpha-value>)",
        // Borders
        border: {
          DEFAULT: "rgb(var(--border) / <alpha-value>)",
          strong: "rgb(var(--border-strong) / <alpha-value>)",
          subtle: "rgb(var(--border) / <alpha-value>)",
          default: "rgb(var(--border-strong) / <alpha-value>)",
        },
        // Text
        fg: {
          DEFAULT: "rgb(var(--fg) / <alpha-value>)",
          muted: "rgb(var(--fg-muted) / <alpha-value>)",
          subtle: "rgb(var(--fg-subtle) / <alpha-value>)",
          dim: "rgb(var(--fg-subtle) / <alpha-value>)",
        },
        text: {
          primary: "rgb(var(--fg) / <alpha-value>)",
          secondary: "rgb(var(--fg-muted) / <alpha-value>)",
          tertiary: "rgb(var(--fg-subtle) / <alpha-value>)",
        },
        // Accent (único tom)
        accent: {
          DEFAULT: "rgb(var(--accent) / <alpha-value>)",
          hover: "rgb(var(--accent-hover) / <alpha-value>)",
          fg: "rgb(var(--accent-fg) / <alpha-value>)",
        },
        // Semantic (score/status numbers)
        success: "rgb(var(--success) / <alpha-value>)",
        warn: "rgb(var(--warn) / <alpha-value>)",
        danger: "rgb(var(--danger) / <alpha-value>)",
        info: "rgb(var(--info) / <alpha-value>)",
        // Tints
        "success-tint": "rgb(var(--success-tint) / <alpha-value>)",
        "warn-tint": "rgb(var(--warn-tint) / <alpha-value>)",
        "danger-tint": "rgb(var(--danger-tint) / <alpha-value>)",
        "info-tint": "rgb(var(--info-tint) / <alpha-value>)",
        // Backward-compat aliases (old class names still resolve)
        edge: "rgb(var(--border) / <alpha-value>)",
        "edge-strong": "rgb(var(--border-strong) / <alpha-value>)",
        neon: "rgb(var(--accent) / <alpha-value>)",
        iris: "rgb(var(--info) / <alpha-value>)",
      },
      fontFamily: {
        sans: ['"Inter Variable"', "ui-sans-serif", "system-ui", "sans-serif"],
        display: ['"Inter Variable"', "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', "ui-monospace", "monospace"],
      },
      fontSize: {
        // DESIGN.md scale: 12 / 13 / 14(base) / 16 / 20 / 28
        xs: ["12px", { lineHeight: "16px" }],
        sm: ["13px", { lineHeight: "18px" }],
        base: ["14px", { lineHeight: "20px" }],
        md: ["16px", { lineHeight: "24px" }],
        lg: ["20px", { lineHeight: "28px" }],
        xl: ["28px", { lineHeight: "34px" }],
      },
      maxWidth: {
        content: "960px",
        "content-wide": "1120px",
      },
      borderRadius: {
        DEFAULT: "8px",
        sm: "6px",
        md: "8px",
        lg: "12px",
        xl: "12px",
      },
      boxShadow: {
        // Cards define-se por borda+fundo; sombra quase nula.
        subtle: "0 1px 2px rgb(0 0 0 / 0.04)",
        sm: "0 1px 2px rgb(0 0 0 / 0.04)",
        md: "0 4px 16px -6px rgb(0 0 0 / 0.10)",
        overlay: "0 12px 32px -8px rgb(0 0 0 / 0.16)",
        glow: "0 1px 2px rgb(0 0 0 / 0.04)",
        "glow-sm": "0 1px 2px rgb(0 0 0 / 0.04)",
        glass: "0 1px 2px rgb(0 0 0 / 0.04)",
      },
    },
  },
  plugins: [],
};
