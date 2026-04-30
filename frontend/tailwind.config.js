/** @type {import('tailwindcss').Config} */
// Design system v0.2 — valeurs extraites des HTML de design/.
// Source de vérité : design/Welcome v2.html, Upload v2.html, etc.
// (Les noms en kebab-case ici matchent les variables CSS dans tokens.css.)
export default {
  content: ['./index.html', './src/**/*.{vue,ts,js}'],
  theme: {
    extend: {
      colors: {
        pond: {
          deep:    '#0E4F6B',
          deeper:  '#062E40',
          mid:     '#3E8AA8',
          light:   '#A9D8E5',
          pale:    '#E6F3F8',
        },
        cream: {
          DEFAULT: '#FAF3E3',
          deep:    '#F2E8CF',
          line:    '#EADFB8',
        },
        duck: {
          DEFAULT: '#FFC93C',
          deep:    '#F2B400',
        },
        champagne: {
          DEFAULT: '#E8C77A',
          pale:    '#F6EAC4',
          deep:    '#C9A347',
        },
        coral: {
          soft:    '#F8D9CF',
          deep:    '#B85A3F',
          line:    '#EDB7A4',
        },
        rec:     '#E14B3F',
        green:   '#5BA85B',
        ink: {
          DEFAULT: '#1F2933',
          soft:    '#5A6B7A',
        },
      },
      fontFamily: {
        // CTA, titres, pseudos
        display: ['Caprasimo', 'Cooper Black', 'Georgia', 'serif'],
        // Texte courant, labels
        sans:    ['"DM Sans"', '-apple-system', 'system-ui', 'sans-serif'],
        // Timestamps, durées, sub-labels
        mono:    ['"DM Mono"', 'ui-monospace', '"SF Mono"', 'Menlo', 'monospace'],
      },
      boxShadow: {
        // Bouton primaire physique (Duck Yellow)
        'btn-primary':  '0 4px 0 #F2B400, 0 10px 24px -8px rgba(242,180,0,.55)',
        'btn-primary-active': '0 1px 0 #F2B400, 0 4px 10px -4px rgba(242,180,0,.4)',
        // Bouton secondaire cream-deep
        'btn-cream':    '0 4px 0 #EADFB8, 0 8px 18px -8px rgba(14,79,107,.18)',
        // Bouton secondaire coral
        'btn-coral':    '0 4px 0 #EDB7A4, 0 10px 20px -8px rgba(184,90,63,.28)',
        // Reroll mid (welcome)
        'reroll':       '0 3px 0 #0E4F6B, 0 8px 18px -6px rgba(14,79,107,.45)',
      },
      borderRadius: {
        // Phone shells, gros containers
        'phone': '36px',
      },
    },
  },
  plugins: [],
}
