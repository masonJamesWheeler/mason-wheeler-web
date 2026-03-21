/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./app/src/**/*.rs'],
  theme: {
    extend: {
      colors: {
        brand: {
          50: '#fff8f0',
          600: '#ea580c',
          700: '#c2410c',
        }
      }
    },
  },
  plugins: [],
}
