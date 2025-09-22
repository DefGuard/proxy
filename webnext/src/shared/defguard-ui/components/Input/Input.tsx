import { useId, useRef } from 'react';
import './style.scss';
import clsx from 'clsx';
import { isPresent } from '../../utils/isPresent';
import { mergeRefs } from '../../utils/mergeRefs';
import { FieldError } from '../FieldError/FieldError';
import { FieldLabel } from '../FieldLabel/FieldLabel';
import type { InputProps } from './types';

export const Input = ({
  value,
  error,
  label,
  ref,
  name,
  size = 'default',
  type = 'text',
  required = false,
  disabled = false,
  canError = false,
  errorPadding = false,
  onChange,
  onBlur,
  onFocus,
  placeholder,
}: InputProps) => {
  const innerRef = useRef<HTMLInputElement>(null);
  const id = useId();
  return (
    <div className="input spacer">
      <div
        className={clsx('inner', {
          error: canError && errorPadding,
          disabled,
        })}
      >
        {isPresent(label) && <FieldLabel required={required} text={label} htmlFor={id} />}
        <div
          className={clsx('track', `size-${size}`, {
            disabled,
          })}
          onClick={() => {
            innerRef.current?.focus();
          }}
        >
          <input
            ref={mergeRefs([ref, innerRef])}
            id={id}
            value={value ?? ''}
            name={name}
            type={type}
            placeholder={placeholder}
            disabled={disabled}
            onFocus={onFocus}
            onBlur={onBlur}
            onChange={(e) => {
              onChange?.(e.target.value);
            }}
          />
        </div>
        {canError && <FieldError error={disabled ? undefined : error} />}
      </div>
    </div>
  );
};
