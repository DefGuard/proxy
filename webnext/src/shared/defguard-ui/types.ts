import type { SVGProps } from 'react';

export const Direction = {
  UP: 'up',
  DOWN: 'down',
  LEFT: 'left',
  RIGHT: 'right',
} as const;

export type Direction = (typeof Direction)[keyof typeof Direction];

export type TSSvgProps = SVGProps<SVGSVGElement>;

export const ThemeVariable = {
  BgDefault: 'var(--bg-default)',
  BgDark: 'var(--bg-dark)',
  BgWhite: 'var(--bg-white)',
  BgFaded: 'var(--bg-faded)',
  BgDisabled: 'var(--bg-disabled)',
  BgMuted: 'var(--bg-muted)',
  BgEmphasis: 'var(--bg-emphasis)',
  BgActive: 'var(--bg-active)',
  BgInverted: 'var(--bg-inverted)',
  BgAction: 'var(--bg-action)',
  BgActionEmphasis: 'var(--bg-action-emphasis)',
  BgActionFaded: 'var(--bg-action-faded)',
  BgActionMuted: 'var(--bg-action-muted)',
  BgCriticalEmphasis: 'var(--bg-critical-emphasis)',
  BgCritical: 'var(--bg-critical)',
  BgCriticalFaded: 'var(--bg-critical-faded)',
  BgCriticalMuted: 'var(--bg-critical-muted)',
  BgSuccess: 'var(--bg-success)',
  BgWarning: 'var(--bg-warning)',
  BorderAction: 'var(--border-action)',
  BorderActionDisabled: 'var(--border-action-disabled)',
  BorderDefault: 'var(--border-default)',
  BorderDisabled: 'var(--border-disabled)',
  BorderEmphasis: 'var(--border-emphasis)',
  BorderMuted: 'var(--border-muted)',
  BorderFaded: 'var(--border-faded)',
  BorderCritical: 'var(--border-critical)',
  BorderSuccess: 'var(--border-success)',
  BorderWarning: 'var(--border-warning)',
  FgAction: 'var(--fg-action)',
  FgActionEmphasis: 'var(--fg-action-emphasis)',
  FgActionMuted: 'var(--fg-action-muted)',
  FgAttention: 'var(--fg-attention)',
  FgCritical: 'var(--fg-critical)',
  FgCriticalMuted: 'var(--fg-critical-muted)',
  FgDefault: 'var(--fg-default)',
  FgFaded: 'var(--fg-faded)',
  FgNeutral: 'var(--fg-neutral)',
  FgMuted: 'var(--fg-muted)',
  FgDisabled: 'var(--fg-disabled)',
  FgSuccess: 'var(--fg-success)',
  FgSuccessMuted: 'var(--fg-success-muted)',
  FgCode: 'var(--fg-code)',
  FgInverted: 'var(--fg-inverted)',
  FgWhite: 'var(--fg-white)',
  FgWhiteTransparent: 'var(--fg-white-transparent)',
  FgBlueTransparent2: 'var(--fg-blue-transparent-2)',
} as const;

export type ThemeVariableValue = (typeof ThemeVariable)[keyof typeof ThemeVariable];

export const TextStyle = {
  TBodyXxs600: 'var(--t-body-xxs-600)',
  TBodyXxs400: 'var(--t-body-xxs-400)',
  TBodyXs600: 'var(--t-body-xs-600)',
  TBodyXs500: 'var(--t-body-xs-500)',
  TBodyXs400: 'var(--t-body-xs-400)',
  TBodySm600: 'var(--t-body-sm-600)',
  TBodySm500: 'var(--t-body-sm-500)',
  TBodySm400: 'var(--t-body-sm-400)',
  TBodyPrimary400: 'var(--t-body-primary-400)',
  TBodyPrimary600: 'var(--t-body-primary-600)',
  TBodyPrimary500: 'var(--t-body-primary-500)',
  TTitleH5: 'var(--t-title-h5)',
  TTitleH4: 'var(--t-title-h4)',
  TTitleH3: 'var(--t-title-h3)',
  TTitleH2: 'var(--t-title-h2)',
  TTitleH1: 'var(--t-title-h1)',
} as const;

export type TextStyleValue = (typeof TextStyle)[keyof typeof TextStyle];

export const BorderRadius = {
  Sm: 'var(--radius-sm)',
  Md: 'var(--radius-md)',
  Lg: 'var(--radius-lg)',
  Xl: 'var(--radius-xl)',
  Xxl: 'var(--radius-xxl)',
  Xxxl: 'var(--radius-xxl)',
  Full: 'var(--radius-full)',
} as const;

export type BorderRadiusValue = (typeof BorderRadius)[keyof typeof BorderRadius];

export const Orientation = {
  Horizontal: 'horizontal',
  Vertical: 'vertical',
} as const;

export type OrientationValue = (typeof Orientation)[keyof typeof Orientation];

export const ThemeSize = {
  Xs: 'var(--size-xs)',
  Sm: 'var(--size-sm)',
  Md: 'var(--size-md)',
  Xl: 'var(--size-xl)',
  Xl2: 'var(--size-2xl)',
  Xl3: 'var(--size-3xl)',
  Xl4: 'var(--size-4xl)',
  Xl5: 'var(--size-5xl)',
};

export type ThemeSizeValue = (typeof ThemeSize)[keyof typeof ThemeSize];

export const ThemeSpacing = {
  Xs: 'var(--spacing-xs)',
  Sm: 'var(--spacing-sm)',
  Md: 'var(--spacing-md)',
  Lg: 'var(--spacing-lg)',
  Xl: 'var(--spacing-xl)',
  Xl2: 'var(--spacing-2xl)',
  Xl3: 'var(--spacing-3xl)',
  Xl4: 'var(--spacing-4xl)',
  Xl5: 'var(--spacing-5xl)',
  Xl6: 'var(--spacing-6xl)',
  Xl7: 'var(--spacing-7xl)',
  Xl8: 'var(--spacing-8xl)',
  Xl9: 'var(--spacing-9xl)',
};

export type ThemeSpacingValue = (typeof ThemeSpacing)[keyof typeof ThemeSpacing];
