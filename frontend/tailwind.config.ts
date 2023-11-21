import { type Config } from "tailwindcss";
import { fontFamily } from "tailwindcss/defaultTheme";

export default {
  content: ["./src/**/*.tsx"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["var(--font-commit-mono)", ...fontFamily.mono],
        mono: ["var(--font-commit-mono)", ...fontFamily.mono],
      },
    },
    container: {
      center: true,
      padding: {
        DEFAULT: "3rem",
        md: "2rem",
      },
      screens: {
        "2xl": "1400px",
      },
    },
  },
  plugins: [],
} satisfies Config;
