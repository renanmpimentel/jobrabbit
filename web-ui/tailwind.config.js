/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // Semantic tokens mapped to CSS variables with alpha support
        bg: "rgb(var(--bg) / <alpha-value>)",
        surface: "rgb(var(--surface) / <alpha-value>)",
        "surface-2": "rgb(var(--surface-2) / <alpha-value>)",
        border: "rgb(var(--border) / <alpha-value>)",
        fg: {
          DEFAULT: "rgb(var(--fg) / <alpha-value>)",
          muted: "rgb(var(--fg-muted) / <alpha-value>)",
          subtle: "rgb(var(--fg-subtle) / <alpha-value>)",
          dim: "rgb(var(--fg-subtle) / <alpha-value>)", // backward compat alias
        },
        accent: {
          DEFAULT: "rgb(var(--accent) / <alpha-value>)",
          fg: "rgb(var(--accent-fg) / <alpha-value>)",
        },
        success: "rgb(var(--success) / <alpha-value>)",
        warn: "rgb(var(--warn) / <alpha-value>)",
        danger: "rgb(var(--danger) / <alpha-value>)",
        info: "rgb(var(--info) / <alpha-value>)",
        // Backward compat aliases for old color names
        edge: "rgb(var(--border) / <alpha-value>)",
        neon: "rgb(var(--accent) / <alpha-value>)",
        iris: "rgb(var(--info) / <alpha-value>)",
        ink: {
          900: "rgb(var(--bg) / <alpha-value>)",
          850: "rgb(var(--surface) / <alpha-value>)",
          800: "rgb(var(--surface-2) / <alpha-value>)",
          750: "rgb(var(--surface-2) / <alpha-value>)",
          700: "rgb(var(--surface-2) / <alpha-value>)",
          600: "rgb(var(--surface-2) / <alpha-value>)",
        },
        "edge-strong": "rgb(var(--border) / 0.2)",
      },
      fontFamily: {
        sans: ['"Hanken Grotesk Variable"', "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', "ui-monospace", "monospace"],
      },
      borderRadius: {
        lg: "10px",
        xl: "14px",
      },
      boxShadow: {
        sm: "0 1px 2px rgb(0 0 0 / 0.06)",
        md: "0 6px 20px -6px rgb(0 0 0 / 0.12)",
        // Backward compat for glow styles (map to soft shadow)
        glow: "0 1px 2px rgb(0 0 0 / 0.06)",
        "glow-sm": "0 1px 2px rgb(0 0 0 / 0.06)",
        glass: "0 1px 2px rgb(0 0 0 / 0.06)",
      },
    },
  },
  plugins: [],
};
