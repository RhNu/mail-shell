import {
  defaultAssetName,
  defineConfig,
  type AssetType,
  type ResolvedAssetSize,
} from '@vite-pwa/assets-generator/config';

const appleTouchIconSize = 180;
// The object form lets assetName distinguish the conventional filename alias.
const conventionalAppleTouchIconSize = {
  width: appleTouchIconSize,
  height: appleTouchIconSize,
};

function assetName(type: AssetType, size: ResolvedAssetSize) {
  if (
    type === 'apple' &&
    typeof size.original !== 'number' &&
    size.width === appleTouchIconSize &&
    size.height === appleTouchIconSize
  ) {
    return 'apple-touch-icon.png';
  }

  return defaultAssetName(type, size);
}

export default defineConfig({
  preset: {
    assetName,
    transparent: {
      sizes: [64, 192, 512],
      favicons: [[48, 'favicon.ico']],
    },
    maskable: {
      sizes: [512],
      padding: 0,
      resizeOptions: {
        background: '#111111',
      },
    },
    apple: {
      sizes: [appleTouchIconSize, conventionalAppleTouchIconSize],
      padding: 0,
      resizeOptions: {
        background: '#111111',
      },
    },
  },
  images: ['public/favicon.svg'],
});
