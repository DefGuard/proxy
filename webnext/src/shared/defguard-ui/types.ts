import type { SVGProps } from 'react';

export const Direction = {
  UP: 'up',
  DOWN: 'down',
  LEFT: 'left',
  RIGHT: 'right',
} as const;

export type Direction = (typeof Direction)[keyof typeof Direction];

export type TSSvgProps = SVGProps<SVGSVGElement>;

export const ThemeKey = {
  BgDefault: 'var(--theme-bg-default)',
  BgDark: 'var(--theme-bg-dark)',
  BgWhite: 'var(--theme-bg-white)',
  BgFaded: 'var(--theme-bg-faded)',
  BgDisabled: 'var(--theme-bg-disabled)',
  BgMuted: 'var(--theme-bg-muted)',
  BgEmphasis: 'var(--theme-bg-emphasis)',
  BgActive: 'var(--theme-bg-active)',
  BgInverted: 'var(--theme-bg-inverted)',
  BgAction: 'var(--theme-bg-action)',
  BgActionEmphasis: 'var(--theme-bg-action-emphasis)',
  BgActionFaded: 'var(--theme-bg-action-faded)',
  BgActionMuted: 'var(--theme-bg-action-muted)',
  BgCriticalEmphasis: 'var(--theme-bg-critical-emphasis)',
  BgCritical: 'var(--theme-bg-critical)',
  BgCriticalFaded: 'var(--theme-bg-critical-faded)',
  BgCriticalMuted: 'var(--theme-bg-critical-muted)',
  BgSuccess: 'var(--theme-bg-success)',
  BgWarning: 'var(--theme-bg-warning)',
  BorderAction: 'var(--theme-border-action)',
  BorderActionDisabled: 'var(--theme-border-action-disabled)',
  BorderDefault: 'var(--theme-border-default)',
  BorderDisabled: 'var(--theme-border-disabled)',
  BorderEmphasis: 'var(--theme-border-emphasis)',
  BorderMuted: 'var(--theme-border-muted)',
  BorderFaded: 'var(--theme-border-faded)',
  BorderCritical: 'var(--theme-border-critical)',
  BorderSuccess: 'var(--theme-border-success)',
  BorderWarning: 'var(--theme-border-warning)',
  FgAction: 'var(--theme-fg-action)',
  FgActionEmphasis: 'var(--theme-fg-action-emphasis)',
  FgActionMuted: 'var(--theme-fg-action-muted)',
  FgAttention: 'var(--theme-fg-attention)',
  FgCritical: 'var(--theme-fg-critical)',
  FgCriticalMuted: 'var(--theme-fg-critical-muted)',
  FgDefault: 'var(--theme-fg-default)',
  FgFaded: 'var(--theme-fg-faded)',
  FgNeutral: 'var(--theme-fg-neutral)',
  FgMuted: 'var(--theme-fg-muted)',
  FgDisabled: 'var(--theme-fg-disabled)',
  FgSuccess: 'var(--theme-fg-success)',
  FgSuccessMuted: 'var(--theme-fg-success-muted)',
  FgCode: 'var(--theme-fg-code)',
  FgInverted: 'var(--theme-fg-inverted)',
  FgWhite: 'var(--theme-fg-white)',
  FgWhiteTransparent: 'var(--theme-fg-white-transparent)',
  FgBlueTransparent2: 'var(--theme-fg-blue-transparent-2)',
} as const;

export type ThemeKey = (typeof ThemeKey)[keyof typeof ThemeKey];
