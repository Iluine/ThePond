import {
  defineConfig,
  minimal2023Preset as preset,
} from '@vite-pwa/assets-generator/config'

/**
 * PWA assets generation config.
 *
 * Source: public/favicon.svg (Duck Yellow + cream safe zone 80%).
 * Run `npm run icons` to regenerate the PNG/ICO outputs in public/.
 *
 * The generated files are committed to the repo so build is reproducible
 * without re-running this step.
 */
export default defineConfig({
  preset: {
    ...preset,
    apple: {
      ...preset.apple,
      // Cream halo around the icon on iOS — matches the home screen
      // visual where icons get rounded square clipping anyway.
      padding: 0.3,
      resizeOptions: {
        ...preset.apple.resizeOptions,
        background: '#FAF3E3',
      },
    },
  },
  images: ['public/favicon.svg'],
})
