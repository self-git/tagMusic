/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,ts}"],
  theme: {
    extend: {
      // 语义颜色 token（取值见 src/style.css，随 prefers-color-scheme 翻转）
      colors: {
        base: "rgb(var(--c-base) / <alpha-value>)",
        surface: "rgb(var(--c-surface) / <alpha-value>)",
        field: "rgb(var(--c-field) / <alpha-value>)",
        elevated: "rgb(var(--c-elevated) / <alpha-value>)",
        line: "rgb(var(--c-line) / <alpha-value>)",
        edge: "rgb(var(--c-edge) / <alpha-value>)",
        strong: "rgb(var(--c-strong) / <alpha-value>)",
        muted: "rgb(var(--c-muted) / <alpha-value>)",
        faint: "rgb(var(--c-faint) / <alpha-value>)",
        dim: "rgb(var(--c-dim) / <alpha-value>)",
        accent: {
          DEFAULT: "rgb(var(--c-accent) / <alpha-value>)",
          hover: "rgb(var(--c-accent-hover) / <alpha-value>)",
          fg: "rgb(var(--c-accent-fg) / <alpha-value>)",
        },
        info: {
          bg: "rgb(var(--c-info-bg) / <alpha-value>)",
          edge: "rgb(var(--c-info-edge) / <alpha-value>)",
          fg: "rgb(var(--c-info-fg) / <alpha-value>)",
        },
        success: {
          DEFAULT: "rgb(var(--c-success) / <alpha-value>)",
          hover: "rgb(var(--c-success-hover) / <alpha-value>)",
          bg: "rgb(var(--c-success-bg) / <alpha-value>)",
          edge: "rgb(var(--c-success-edge) / <alpha-value>)",
          fg: "rgb(var(--c-success-fg) / <alpha-value>)",
        },
        warning: {
          bg: "rgb(var(--c-warning-bg) / <alpha-value>)",
          edge: "rgb(var(--c-warning-edge) / <alpha-value>)",
          fg: "rgb(var(--c-warning-fg) / <alpha-value>)",
        },
        danger: {
          bg: "rgb(var(--c-danger-bg) / <alpha-value>)",
          edge: "rgb(var(--c-danger-edge) / <alpha-value>)",
          fg: "rgb(var(--c-danger-fg) / <alpha-value>)",
        },
      },
    },
  },
  plugins: [],
};
