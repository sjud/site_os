/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["*.html", "./src/**/*.rs"],
  theme: {
    extend: {
      keyframes: {
        jump: {
          '0%' : {
            transform: 'translateY(0px)'
          },
          '45%' : {
            transform: 'translateY(-19px)'
          },
          '50%' : {
            transform: 'translateY(-20px)'
          },
          '55%' : {
            transform: 'translateY(-19px)'
          },
          '100%' : {
            transform: 'translateY(0px)'
          },
        }
      },
      animation: {
        jump: 'jump 0.8s 1 linear',
      },
    },
  },
  plugins: [],
}
