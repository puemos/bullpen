/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html"],
  theme: {
    extend: {
      colors: {
        paper: "#FAFAFA",
        ink: "#0B0B0C",
        muted: "#6B6B72",
        rule: "#D9D9DE",
        lavender: {
          DEFAULT: "#E8DEF4",
          deep: "#C8B8DF",
        },
        violet: "#6B3F8A",
        plum: "#3A1F4E",
        magenta: "#8B3D6B",
        gold: "#F2C542",
        amber: "#D4A017",
        stance: {
          bullish: "#059669",
          bearish: "#DC2626",
          mixed: "#D97706",
          neutral: "#71717A",
        },
      },
      fontFamily: {
        sans: ['"Inter"', "system-ui", "sans-serif"],
        serif: ['"Source Serif 4"', '"Source Serif Pro"', "Georgia", "serif"],
        display: ['"Inter Tight"', '"Inter"', "sans-serif"],
        mono: ['"JetBrains Mono"', "ui-monospace", "monospace"],
      },
      borderRadius: {
        DEFAULT: "0",
        none: "0",
        sm: "0",
        md: "0",
        lg: "0",
        xl: "0",
        "2xl": "0",
        "3xl": "0",
        full: "9999px",
      },
    },
  },
  plugins: [],
};
