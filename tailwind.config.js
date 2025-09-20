/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f0f9f4',
          100: '#dcf4e6',
          500: '#18A058',
          600: '#16a085',
          700: '#0d8043',
        },
        info: {
          500: '#2080F0',
        },
        warning: {
          500: '#F0A020',
        },
        error: {
          500: '#D03050',
        }
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
      }
    },
  },
  plugins: [],
}