/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'riptide-blue': '#0ea5e9',
        'riptide-dark': '#0f172a',
      }
    },
  },
  plugins: [],
}
