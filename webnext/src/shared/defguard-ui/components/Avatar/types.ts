export const AvatarSize = {
  Default: 'default',
  Small: 'small',
  Big: 'big',
} as const;

export type AvatarSizeValue = (typeof AvatarSize)[keyof typeof AvatarSize];
