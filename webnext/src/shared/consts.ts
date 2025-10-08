export const motionTransitionStandard = {
  type: 'tween',
  ease: 'easeOut',
  duration: 0.16,
} as const;

export const externalLink = {
  client: {
    desktop: {
      linux: {
        arch: 'https://aur.archlinux.org/packages/defguard-client',
      },
    },
    mobile: {
      apple: 'https://apps.apple.com/us/app/defguard-vpn-client/id6748068630',
      google: 'https://play.google.com/store/apps/details?id=net.defguard.mobile',
    },
  },
};
