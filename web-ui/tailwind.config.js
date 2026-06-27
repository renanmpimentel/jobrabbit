/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // Mission Control — ink profundo + neon mint.
        ink: {
          900: "#070b14",
          850: "#0a1020",
          800: "#0e1424",
          750: "#121a2e",
          700: "#172037",
          600: "#1e2942",
        },
        edge: "rgba(255,255,255,0.08)",
        "edge-strong": "rgba(255,255,255,0.16)",
        fg: {
          DEFAULT: "#e8edf7",
          muted: "#8b97b0",
          dim: "#5b6781",
        },
        neon: {
          DEFAULT: "#5BF0A5",
          dim: "#36d488",
        },
        iris: "#8B9CFF",
        warn: "#FBBF24",
        danger: "#FB6F6F",
      },
      fontFamily: {
        display: ['"Bricolage Grotesque Variable"', "ui-sans-serif", "sans-serif"],
        sans: ['"Hanken Grotesk Variable"', "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', "ui-monospace", "monospace"],
      },
      borderRadius: {
        xl: "14px",
        "2xl": "18px",
      },
      boxShadow: {
        glow: "0 0 0 1px rgba(91,240,165,0.35), 0 0 26px -6px rgba(91,240,165,0.5)",
        "glow-sm": "0 0 18px -6px rgba(91,240,165,0.55)",
        glass:
          "0 18px 50px -20px rgba(0,0,0,0.7), inset 0 1px 0 0 rgba(255,255,255,0.06)",
      },
      keyframes: {
        pulseGlow: {
          "0%, 100%": { opacity: "1", boxShadow: "0 0 0 0 rgba(91,240,165,0.0)" },
          "50%": { opacity: "0.85", boxShadow: "0 0 14px 2px rgba(91,240,165,0.5)" },
        },
        shimmer: {
          "100%": { transform: "translateX(100%)" },
        },
      },
      animation: {
        pulseGlow: "pulseGlow 2s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
