/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,rs}",
    "index.html"
  ],
  theme: {
    extend: {
      colors: {
        "background": "#131313",
        "accent-1": "#202020",
        "smoky-black": "#0E0606",
        "chocolate-cosmos": "#56131A",
        "cornell-red": "#A4161A",
        "timberwolf": "#DBD5D2",
        "white-smoke": "#F5F3F4",
      }
    },
    fontFamily: {
      "fira-sans": ["Fira Sans", "sans-serif"]
    }
  },
  plugins: [],
}

