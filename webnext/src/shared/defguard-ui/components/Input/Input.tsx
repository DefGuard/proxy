import { type HTMLInputTypeAttribute, useId, useMemo, useRef, useState } from 'react';
import './style.scss';
import clsx from 'clsx';
import { isPresent } from '../../utils/isPresent';
import { mergeRefs } from '../../utils/mergeRefs';
import { FieldBox } from '../FieldBox/FieldBox';
import { FieldError } from '../FieldError/FieldError';
import { FieldLabel } from '../FieldLabel/FieldLabel';
import type { IconKindValue } from '../Icon/icon-types';
import type { InputProps } from './types';

export const Input = ({
  value,
  error,
  label,
  ref,
  name,
  placeholder,
  boxProps,
  testId,
  onChange,
  onBlur,
  onFocus,
  size = 'default',
  type = 'text',
  required = false,
  disabled = false,
  autocomplete = 'off',
}: InputProps) => {
  const isPassword = useMemo(() => type === 'password', [type]);

  const [inputTypeInner, setInputType] = useState<HTMLInputTypeAttribute>(type);
  const innerRef = useRef<HTMLInputElement>(null);
  const id = useId();

  const interactionIconRight = useMemo((): IconKindValue | undefined => {
    if (isPassword) {
      if (inputTypeInner === 'password') {
        return 'show';
      } else {
        return 'hide';
      }
    }
  }, [isPassword, inputTypeInner]);

  return (
    <div className="input spacer">
      <div
        className={clsx('inner', {
          disabled,
        })}
      >
        {isPresent(label) && <FieldLabel required={required} text={label} htmlFor={id} />}
        <FieldBox
          className="input-track"
          error={!disabled && isPresent(error)}
          disabled={disabled}
          size={size}
          onClick={() => {
            innerRef.current?.focus();
          }}
          iconRight={interactionIconRight}
          onInteractionClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            if (isPassword) {
              setInputType((s) => {
                if (s === 'password') {
                  return 'text';
                }
                return 'password';
              });
            }
          }}
          {...boxProps}
        >
          <input
            ref={mergeRefs([ref, innerRef])}
            id={id}
            autoComplete={autocomplete}
            data-testid={testId}
            value={value ?? ''}
            name={name}
            type={inputTypeInner}
            placeholder={placeholder}
            disabled={disabled}
            onFocus={onFocus}
            onBlur={onBlur}
            onChange={(e) => {
              onChange?.(e.target.value);
            }}
          />
        </FieldBox>
        <FieldError error={disabled ? undefined : error} />
      </div>
    </div>
  );
};
