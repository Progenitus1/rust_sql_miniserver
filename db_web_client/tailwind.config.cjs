module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        'poppins': ['"Poppins"', 'sans-serif'],
      },
    },
    colors: {
      transparent: "transparent",
      current: "currentColor",
      black: "#000",
      white: "#fff",
      'dark-gray': "#171717",
      'gray': "#1c1c1e",
      blue: '#6366f1',
      'light-gray': '#f5f5f5',
      orange: '#d99129',
      red: '#8B0000',
      green: '#98bf44'
    }
  },
  plugins: [],
}
