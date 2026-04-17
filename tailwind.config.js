/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,ts,js}"],
  theme: {
    extend: {
      colors: {
        bg:      "#080d1a",
        bg2:     "#0d1525",
        bg3:     "#111c30",
        card:    "#0f1829",
        card2:   "#14203a",
        border:  "#1e2d4a",
        border2: "#263756",
        text:    "#dce8ff",
        text2:   "#6478a0",
        text3:   "#3a4e72",
        accent:  "#4d88f5",
        accent2: "#7aaaff",
        green:   "#2ecc82",
        yellow:  "#f0c030",
        orange:  "#e87848",
        red:     "#e05050",
        purple:  "#9470f0",
        teal:    "#24b0a8",
        pink:    "#e85598",
        gold:    "#ffd700",
      },
      fontFamily: {
        sans: ['"Segoe UI"', "system-ui", "sans-serif"],
        mono: ['"Cascadia Code"', '"Consolas"', "monospace"],
      },
    },
  },
  plugins: [],
};
