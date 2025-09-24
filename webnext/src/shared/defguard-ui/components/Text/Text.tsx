import type {
  PolymorphicForwardedRef,
  PolymorphicProps,
} from '@axa-ch/react-polymorphic-types';
import type React from 'react';
import { createElement, type ElementType } from 'react';
import type { TextStyleValue } from '../../types';

type TextDefault = 'p';
type TextAllowed =
  | TextDefault
  | 'span'
  | 'strong'
  | 'em'
  | 'small'
  | 'label'
  | 'code'
  | 'kbd'
  | 'mark'
  | 'sup'
  | 'sub'
  | 'time'
  | 'abbr'
  | 'q'
  | 'cite'
  | 'del'
  | 'ins'
  | 'pre'
  | 'blockquote'
  | 'h1'
  | 'h2'
  | 'h3'
  | 'h4'
  | 'h5'
  | 'h6';

type OwnProps<T extends TextAllowed> = React.ComponentPropsWithoutRef<T> & {
  font: TextStyleValue;
};

type TextProps<T extends TextAllowed = TextDefault> = PolymorphicProps<
  OwnProps<T>,
  T,
  TextAllowed
> & {
  ref?: PolymorphicForwardedRef<T>;
};

export const AppText = <T extends TextAllowed = TextDefault>({
  as,
  children,
  style,
  font,
  ...rest
}: TextProps<T>) => {
  const Component = (as ?? ('p' satisfies ElementType)) as ElementType;
  const computedStyle = { ...style, font };
  return createElement(Component, { ...rest, style: computedStyle }, children);
};
