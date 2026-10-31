/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        bio: {
          bg: '#08090d',
          surface: '#0f1118',
          card: '#141824',
          border: '#1f2538',
          text: '#f1f5f9',
          muted: '#94a3b8',
          stem: '#10b981',        // Emerald green
          progenitor: '#06b6d4',  // Cyan
          specialized: '#a855f7', // Electric purple
          apoptosis: '#ef4444',   // Red
          pheromone: '#f59e0b',   // Gold
          shield: '#f97316',      // Orange
        }
      },
      animation: {
        'pulse-glow': 'pulseGlow 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'pheromone-flow': 'pheromoneFlow 1.5s linear infinite',
      },
      keyframes: {
        pulseGlow: {
          '0%, 100%': { opacity: '0.4', transform: 'scale(1)' },
          '50%': { opacity: '0.9', transform: 'scale(1.08)' },
        }
      }
    },
  },
  plugins: [],
}
