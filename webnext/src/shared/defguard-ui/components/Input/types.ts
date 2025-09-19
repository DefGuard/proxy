import type { HTMLInputTypeAttribute, Ref } from 'react';

export type InputProps = {
  value: string | null;
  type?: HTMLInputTypeAttribute;
  ref?: Ref<HTMLInputElement>;
  error?: string;
  canError?: boolean;
};
