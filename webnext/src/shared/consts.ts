export const motionTransitionStandard = {
  type: 'tween',
  ease: 'easeOut',
  duration: 0.16,
} as const;

export const externalLink = {
  client: {
    desktop: {
      windows:
        'https://github.com/DefGuard/client/releases/download/v1.5.1/defguard-client_1.5.1_x64_en-US.exe',
      linux: {
        arch: 'https://aur.archlinux.org/packages/defguard-client',
        deb: {
          amd: 'https://github.com/DefGuard/client/releases/download/v1.5.1/defguard-client_1.5.1_amd64.deb',
          arm: 'https://github.com/DefGuard/client/releases/download/v1.5.1/dg-linux-aarch64-v1.5.1.deb',
        },
        rpm: {
          amd: 'https://github.com/DefGuard/client/releases/download/v1.5.1/defguard-client-1.5.1-1.x86_64.rpm',
          arm: 'https://github.com/DefGuard/client/releases/download/v1.5.1/defguard-client-1.5.1-1.aarch64.rpm',
        },
      },
      macos: {
        intel:
          'https://github.com/DefGuard/client/releases/download/v1.5.1/defguard-x86_64-apple-darwin-1.5.1.pkg',
        arm: 'https://github.com/DefGuard/client/releases/download/v1.5.1/defguard-aarch64-apple-darwin-1.5.1.pkg',
      },
    },
    mobile: {
      apple: 'https://apps.apple.com/us/app/defguard-vpn-client/id6748068630',
      google: 'https://play.google.com/store/apps/details?id=net.defguard.mobile',
    },
  },
};
