export const BadgeVariant = {
  Neutral: 'neutral',
  Success: 'success',
  Critical: 'critical',
  Warning: 'warning',
} as const;

export type BadgeVariantValue = (typeof BadgeVariant)[keyof typeof BadgeVariant];
