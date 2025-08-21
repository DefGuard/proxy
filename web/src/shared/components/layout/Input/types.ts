import type { HTMLMotionProps } from 'motion/react';
import type { ReactNode } from 'react';

export type InputFloatingErrors = {
  title: string;
  errorMessages: string[];
};

export interface InputProps extends HTMLMotionProps<'input'> {
  required?: boolean;
  invalid?: boolean;
  label?: string | ReactNode;
  disableOuterLabelColon?: boolean;
  errorMessage?: string;
  floatingErrors?: InputFloatingErrors;
  disposable?: boolean;
  disposeHandler?: (v?: unknown) => void;
}
